import subprocess
from pathlib import Path
import os
import shutil
from typing import Literal
import colorama
import re

MY_DIR = Path(__file__).parent.absolute()

ORFS = os.environ.get('ORFS')
if ORFS == '':
    raise RuntimeError('ORFS environmental variable is not set')
ORFS = Path(ORFS).absolute()

OPEN_STA=os.environ.get('OPENSTA', ORFS / 'tools/OpenROAD/src/sta/app/sta')
OPEN_STA=Path(OPEN_STA).absolute()

IVERILOG=os.environ.get('IVERILOG', 'iverilog')
TRACE2POWER=os.environ.get('TRACE2POWER', 'trace2power')

class TestError(Exception):
    def __init__(self, message: str):
        super().__init__(message)

    def pretty(self):
        return f'{colorama.Fore.RED}{self.args[0]}{colorama.Fore.RESET}'

class Project:
    def __init__(
        self,
        directory: Path,
        name: str,
        testbench_sources: list[str],
        rtl_sources: list[str],
        clk_freq: float,
        top: str,
        scope: str
    ):
        self.directory = directory.absolute()
        self.name = name
        self.clk_freq = clk_freq
        self.top = top
        self.scope = scope
        self.testbench_sources = testbench_sources
        self.rtl_sources = rtl_sources
        self.pdk = 'sky130hd' #TODO: glob for .lib files in sta.tcl to make it reconfigurable

    def _step_info(self, txt: str) -> None:
        print(f'{colorama.Fore.CYAN}{txt}{colorama.Fore.RESET}', flush=True)

    @property
    def out(self) -> Path:
        return self.directory / 'out'

    def write_config(self):
        self._step_info('Writing config.mk')
        with open(self.out / 'config.mk', 'w+') as f:
            f.writelines([
                f'export PLATFORM = {self.pdk}\n',
                f'export DESIGN_NICKNAME = {self.name}\n',
                f'export DESIGN_NAME = {self.name}\n',
                 'export VERILOG_FILES = ' + \
                    ' '.join(f'$(PROJECT_DIR)/{s}' for s in self.rtl_sources) + '\n',
                f'export SDC_FILE = $(PROJECT_DIR)/constraints.sdc\n'
            ])


    def simulate_testbench(self):
        self._step_info('Simulating testbench with Icarus Verilog')

        vvp_file = self.out / (self.name + '.vvp')
        cp = subprocess.run([
            IVERILOG,
            *(self.directory / f for f in self.testbench_sources),
            *(self.directory / f for f in self.rtl_sources),
            '-g2012',
            '-o', vvp_file
        ])
        if cp.returncode != 0:
            raise TestError(f'iverilog exited with non-zero code: {cp.returncode}')
        cp = subprocess.run(['vvp', vvp_file], cwd=self.out)
        if cp.returncode != 0:
            raise TestError(f'vpp exited with non-zero code: {cp.returncode}')

    def synthesize_design(self):
        self._step_info('Synthesising design with ORFS')
        subprocess.run(
            args=[
            'make',
            '-C', str(ORFS / 'flow'),
            f'PROJECT_DIR={self.directory}',
            f'DESIGN_CONFIG={str(self.out / "config.mk")}',
            'clean_all'
            ],
            stdout=subprocess.DEVNULL
        )
        cp = subprocess.run([
            'make',
            '-C', str(ORFS / 'flow'),
            f'PROJECT_DIR={self.directory}',
            f'DESIGN_CONFIG={str(self.out / "config.mk")}',
            'synth'
        ])
        if cp.returncode != 0:
            raise TestError(f'make synth exited with non-zero code: {cp.returncode}')
        shutil.copy(
            str(ORFS / f'flow/results/sky130hd/{self.name}/base/1_synth.v'),
            str(self.directory / f'out/{self.name}_synth.v')
        )

    def process_vcd(self, output_format: Literal['tcl', 'saif']):
        self._step_info(f'Converting VCD with trace2power to {output_format.upper()}')

        with open(self.directory / f'out/{self.name}.{output_format}', 'w+') as f:
            cp = subprocess.run(
                args=[
                    TRACE2POWER,
                    str(self.directory / 'out' / (self.name + '.vcd')),
                    '--clk-freq', str(self.clk_freq),
                    '--output-format', output_format
                ],
                stdout=f
            )
            if cp.returncode != 0:
                raise TestError(f'trace2power exited with non-zero code: {cp.returncode}')

    def create_power_reports(self, *activity_formats: Literal['vcd', 'tcl', 'saif']):
        self._step_info('Performing power analysis with OpenSTA')

        for activity_fmt in activity_formats:
            pins_annotated = False
            with subprocess.Popen(
                args=[str(OPEN_STA), '-exit', str(MY_DIR / 'sta.tcl')],
                env={
                    'ORFS': str(ORFS),
                    'DESIGN': str(self.name),
                    'DESIGN_TOP': str(self.top),
                    'PROJECT_DIR': str(self.directory),
                    'DESIGN_SCOPE': str(self.scope),
                    'POWER_ACTIVITY_FMT': activity_fmt
                },
                stdout=subprocess.PIPE,
                bufsize=1,
                universal_newlines=True
            ) as p:
                for line in p.stdout:
                    m = re.match('Annotated ([0-9]+) pin activities.', line.strip())
                    if m is not None:
                        print(line, end='', flush=True)
                        if m.group(1) == '0':
                            raise TestError('Zero pins were annotated')
                        pins_annotated = True
            # TODO: The version of OpenSTA (aa598a2) currently included in OpenROAD (2978ae6) does
            # not report the number of activities read from a SAIF file. This has been added in
            # b0bd2d8. Once this makes its way to OpenROAD, this new version should be included in
            # an updated version of CI image and the list of activity formats in this conditional
            # statement should be expanded to include 'saif'.
            if activity_fmt in ['vcd'] and not pins_annotated:
                raise TestError('Pin annotation was not attempted')

            report_path = self.directory / f'out/power_report_{activity_fmt}.txt'

            try:
                with open(report_path) as f:
                    print(
                        f'::: {colorama.Style.BRIGHT}{activity_fmt.upper()}'
                        f'{colorama.Style.RESET_ALL} power report :::',
                        flush=True
                        )
                    print(f.read())
            except FileNotFoundError:
                raise TestError(f'Report "{report_path}" was not generated')

def main():
    examples = [
        Project(
            directory=MY_DIR / 'counter',
            name='counter',
            testbench_sources=['counter_tb.v'],
            rtl_sources=['counter.v'],
            clk_freq=500000000,
            top='counter',
            scope='counter_tb/counter0'
        ),
        Project(
            directory=MY_DIR / 'tristate',
            name='tristate',
            testbench_sources=['tristate_tb.sv'],
            rtl_sources=['tristate.v'],
            clk_freq=500000000,
            top='tristate',
            scope='tristate_tb/tristate0'
        )
    ]

    tests_ok = True

    for example in examples:
        print(
            f'{colorama.Back.CYAN}Tesing example {example.name}{colorama.Back.RESET}', flush=True
        )
        if (example.directory / 'out').exists():
            shutil.rmtree(str(example.directory / 'out'))
        (example.directory / 'out').mkdir()

        try:
            example.write_config()
            example.simulate_testbench()
            example.synthesize_design()
            example.process_vcd('saif')
            example.process_vcd('tcl')
            example.create_power_reports('vcd', 'saif')
        except TestError as e:
            tests_ok = False
            print(f'ERROR ({example.name}): {e.pretty()}')
            continue

        with open(example.directory / 'out/power_report_vcd.txt', 'r') as f:
            vcd_report = f.read()
        with open(example.directory / 'out/power_report_saif.txt', 'r') as f:
            saif_report = f.read()

        if vcd_report == saif_report:
            print(
                f'{colorama.Back.GREEN}Power reports for {example.name} match!'
                f'{colorama.Back.RESET}',
                flush=True
            )
        else:
            print(
                f'{colorama.Back.RED}Power reports for {example.name} differ!{colorama.Back.RESET}',
                flush=True
            )
            tests_ok = False

    if not tests_ok:
        exit(-1)


if __name__ == '__main__':
    main()
