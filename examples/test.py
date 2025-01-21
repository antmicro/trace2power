import subprocess
from pathlib import Path
import os
import shutil
from typing import Literal, Optional
import colorama
import re
from argparse import ArgumentParser
import io

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

    @property
    def msg(self) -> str: return self.args[0]

    def pretty(self):
        return f'{colorama.Fore.RED}{self.msg}{colorama.Fore.RESET}'

class Project:
    def __init__(
        self,
        name: str,
        testbench_sources: list[str],
        rtl_sources: list[str],
        clk_freq: float,
        top: str,
        scope: str,
        directory: Optional[Path] = None,
    ):
        self.directory = directory.absolute() if directory is not None else MY_DIR / name
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
                f'export DESIGN_NAME = {self.top}\n',
                 'export VERILOG_FILES = ' + \
                    ' '.join(f'$(PROJECT_DIR)/{s}' for s in self.rtl_sources) + '\n',
                f'export SDC_FILE = $(PROJECT_DIR)/constraints.sdc\n'
            ])

    def simulate_testbench(self, gate_level: bool = False):
        sim_type_str = 'gate level' if gate_level else 'rtl'
        self._step_info(f'Simulating testbench with Icarus Verilog ({sim_type_str})')

        vvp_file = self.out / (self.name + '.vvp')
        if gate_level:
            pdk_impl = ORFS / f'flow/platforms/{self.pdk}/work_around_yosys/formal_pdk.v'
            cp = subprocess.run([IVERILOG,
                *(self.directory / f for f in self.testbench_sources),
                self.out / f'{self.name}_synth.v',
                str(pdk_impl),
                '-g2012',
                '-o', vvp_file
            ])
        else:
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
            str(ORFS / f'flow/results/{self.pdk}/{self.name}/base/1_synth.v'),
            str(self.directory / f'out/{self.name}_synth.v')
        )

    def process_vcd(self, output_format: Literal['tcl', 'saif']):
        self._step_info(f'Converting VCD with trace2power to {output_format.upper()}')

        args=[
            TRACE2POWER,
            str(self.out / (self.name + '.vcd')),
            '--clk-freq', str(self.clk_freq),
            '--output-format', output_format
        ]
        if output_format == 'tcl':
            args += ['--scope', self.scope.replace('/', '.')]

        with open(self.directory / f'out/{self.name}.{output_format}', 'w+') as f:
            cp = subprocess.run(args=args, stdout=f)
            if cp.returncode != 0:
                raise TestError(f'trace2power exited with non-zero code: {cp.returncode}')

    def create_power_report(self, activity_fmt: Literal['vcd', 'tcl', 'saif', 'null']):
        self._step_info(f'Performing power analysis with OpenSTA for {activity_fmt}')

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
                print(line, end='', flush=True)
                if m is not None:
                    if m.group(1) == '0':
                        raise TestError('Zero pins were annotated')
                    pins_annotated = True
        # WARNING: The version of OpenSTA (aa598a2) currently included in OpenROAD (2978ae6) does
        # not report the number of activities read from a SAIF file. This has been added in
        # b0bd2d8. Once this makes its way to OpenROAD, this new version should be included in
        # an updated version of CI image and the list of activity formats in this conditional
        # statement should be expanded to include 'saif'.
        if activity_fmt in ['vcd', 'saif'] and not pins_annotated:
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

def check_report_match(name: str, reference: str, out, **reports: str) -> bool:
    ref_content = reports.get(reference)
    assert ref_content is not None

    reports_differ = False
    for content in reports.values():
        if content != ref_content:
            reports_differ = True
            break

    if reports_differ:
        print(
            f'{colorama.Back.GREEN}Power reports for {name} match!'
            f'{colorama.Back.RESET}',
            flush=True
        )
    else:
        print(
            f'{colorama.Back.RED}Power reports for {name} differ!{colorama.Back.RESET}',
            flush=True
        )

    for report_name, content in reports.items():
        if report_name == reference:
            continue
        if content == ref_content:
            print(f'* {colorama.Fore.GREEN}✔{colorama.Fore.RESET} {report_name}: PASS', file=out)
        else:
            print(
                f'* {colorama.Fore.RED}✘{colorama.Fore.RESET} {report_name}: FAIL (mismatch)',
                file=out
            )

    return reports_differ


def test_example(example: Project, summary) -> bool:
    print(
        f'{colorama.Back.CYAN}Tesing example {example.name}{colorama.Back.RESET}', flush=True
    )
    if (example.out).exists():
        shutil.rmtree(str(example.out))
    example.out.mkdir()

    test_pass = True
    saif_pass = True
    tcl_pass = True

    def fail(err: TestError, summary_msg: str) -> bool:
        print(f'ERROR ({example.name}): {e.pretty()}')
        print(f'* {colorama.Fore.RED}✘{colorama.Fore.RESET} {summary_msg}', file=summary)
        test_pass = False
        return False

    try:
        example.write_config()
        example.synthesize_design()
        example.simulate_testbench(gate_level=True)
    except TestError as e: return fail(e, f'failure at simulation/synthesis ({e.msg})')

    try: example.create_power_report('null')
    except TestError as e: return fail(e, f'failed to create null report ({e.msg})')
    try: example.create_power_report('vcd')
    except TestError as e: return fail(e, f'failed to create vcd report')

    try: example.process_vcd('saif')
    except TestError as e: saif_pass = fail(e, f'failure in trace2power ({e.msg})')
    if saif_pass:
        try: example.create_power_report('saif')
        except TestError as e: saif_pass = fail(e, f'saif: failed to create report ({e.msg})')
    try: example.process_vcd('tcl')
    except TestError as e: tcl_pass = fail(e, f'failure in trace2power ({e.msg})')
    if tcl_pass:
        try: example.create_power_report('tcl')
        except TestError as e: saif_pass = fail(e, f'tcl: failed to create report ({e.msg})')

    with open(example.directory / 'out/power_report_vcd.txt', 'r') as f:
        vcd_report = f.read()
    with open(example.directory / 'out/power_report_saif.txt', 'r') as f:
        saif_report = f.read()
    with open(example.directory / 'out/power_report_tcl.txt', 'r') as f:
        tcl_report = f.read()

    return check_report_match(
        name=example.name,
        reference='vcd',
        out=summary,
        vcd=vcd_report,
        saif=saif_report,
        tcl=tcl_report,
    )

def main():
    examples = [
        Project(
            name='counter',
            testbench_sources=['counter_tb.v'],
            rtl_sources=['counter.v'],
            clk_freq=500000000,
            top='counter',
            scope='counter_tb/counter0'
        ),
        Project(
            name='tristate',
            testbench_sources=['tristate_tb.sv'],
            rtl_sources=['tristate.v'],
            clk_freq=500000000,
            top='tristate',
            scope='tristate_tb/tristate0'
        ),
        # Warning OpenSTA @ aa598a2 doe snot read glitch activities in SAFI files equivalently
        # to reads from VCDs.
        Project(
            name='hierarchical',
            testbench_sources=['hierarchical_tb.sv'],
            rtl_sources=['hierarchical.sv'],
            clk_freq=500000000,
            top='hierarchical',
            scope='hierarchical_tb/dut'
        ),
        # WARNING: OpenSTA @ aa598a2 does not support combo-only designs
        Project(
            name='tail',
            testbench_sources=['tail.sv'],
            rtl_sources=['big_and.sv'],
            clk_freq=500000000,
            top='big_and',
            scope='tail/dut'
        )
    ]

    arg_parser = ArgumentParser()
    arg_parser.add_argument('project_name', type=str, nargs='*')
    args = arg_parser.parse_args()

    tests_ok = True

    summary = io.StringIO()

    for example in examples:
        if len(args.project_name) != 0 and example.name not in args.project_name:
            continue
        print(f'{example.name}:', file=summary)
        tests_ok &= test_example(example, summary)

    print(f'\nSUMMARY:\n{summary.getvalue()}')
    summary.close()

    if not tests_ok:
        exit(-1)


if __name__ == '__main__':
    main()
