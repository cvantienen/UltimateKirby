"""Split-source assembly and recovery tests; no mod builds or network calls."""
import contextlib
import io
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import sequence_sources as sources
import sequence_transfer as transfer
import build_variants as builder

UNIVERSAL=b'[universal.settings]\nenabled=true\n\n[template.shield]\nname=Shield\nstart_delay=0\nactions=input:zr:0:0:4\n'
KIRBY=b'[kirby.neutral.jab]\nstart_delay=3\nactions=jab:4\ndefault=true\n'
BOWSER=b'[bowser.settings]\n[bowser.roll.chase]\nstart_delay=0\nreaction=auto_roll:30:12\nactions=up_b:4,drift:opponent:1:69\n'


def fixture(root):
    path=root/'sequences.txt';path.write_bytes(UNIVERSAL)
    (root/'fighters').mkdir()
    (root/'fighters/kirby.txt').write_bytes(KIRBY)
    (root/'fighters/bowser.txt').write_bytes(BOWSER)
    return path


class SequenceSourceTests(unittest.TestCase):
    def test_current_bundle_matches_all_79_owned_sources_and_keeps_templates_universal(self):
        data=sources.assemble()
        self.assertEqual(data,sources.GENERATED.read_bytes())
        self.assertTrue(data.endswith(b'\n'))
        self.assertFalse(data.endswith(b'\n\n'))
        self.assertEqual(len(list((sources.SOURCE.parent/'fighters').glob('*.txt'))),79)
        self.assertEqual(set(sources.split_bundle(sources.SOURCE.read_bytes())),{'universal'})
        for file in (sources.SOURCE.parent/'fighters').glob('*.txt'):
            self.assertEqual(set(sources.split_bundle(file.read_bytes())),{file.stem})
        self.assertIn(b'[template.up_b_chase]',data)

    def test_assembly_reads_live_fighter_edits_in_sorted_order_without_altering_properties(self):
        with tempfile.TemporaryDirectory() as tmp:
            path=fixture(Path(tmp));data=sources.assemble(path)
            self.assertLess(data.index(b'[bowser.settings]'),data.index(b'[kirby.neutral.jab]'))
            self.assertIn(KIRBY.rstrip(),data)
            (path.parent/'fighters/kirby.txt').write_bytes(KIRBY.replace(b'start_delay=3',b'start_delay=9'))
            self.assertIn(b'start_delay=9',sources.assemble(path))
            self.assertEqual(path.read_bytes(),UNIVERSAL)

    def test_wrong_owner_universal_fighter_sections_and_oversized_assembly_fail(self):
        with tempfile.TemporaryDirectory() as tmp:
            path=fixture(Path(tmp));file=path.parent/'fighters/kirby.txt'
            file.write_bytes(BOWSER)
            with self.assertRaisesRegex(RuntimeError,'only'):sources.assemble(path)
            file.write_bytes(KIRBY);path.write_bytes(UNIVERSAL+KIRBY)
            with self.assertRaisesRegex(RuntimeError,'only universal'):sources.assemble(path)
            path.write_bytes(UNIVERSAL);file.write_bytes(KIRBY+b'#'+b'x'*sources.MAX_BYTES)
            with self.assertRaises(RuntimeError):sources.assemble(path)

    def test_import_roundtrips_settings_and_templates_and_rejects_concurrent_edits(self):
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp);path=fixture(root);old=sources.assemble(path)
            new=old.replace(b'start_delay=3',b'start_delay=10')
            sources.replace_sources(path,new,root/'backup',expected=old)
            self.assertEqual(sources.assemble(path),new)
            self.assertEqual((root/'backup/kirby.txt').read_bytes(),KIRBY)
            self.assertEqual(sources.split_bundle(path.read_bytes()).keys(),{'universal'})
            with self.assertRaisesRegex(RuntimeError,'changed'):
                sources.replace_sources(path,old,root/'conflict',expected=old)
            self.assertEqual(sources.assemble(path),new)

    def test_failed_import_restores_every_file(self):
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp);path=fixture(root);before=sources.assemble(path);write=sources.write_generated
            def fail_once(data,out):
                if out==root/'fighters/kirby.txt' and b'10' in data:
                    raise OSError('injected write failure')
                write(data,out)
            with patch.object(sources,'write_generated',side_effect=fail_once):
                with self.assertRaisesRegex(OSError,'injected'):
                    sources.replace_sources(path,before.replace(b'start_delay=3',b'start_delay=10'),root/'backup',expected=before)
            self.assertEqual(sources.assemble(path),before)

    def test_local_transfer_reimports_split_sources_with_backups_and_readback(self):
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp);path=fixture(root);incoming=root/'download.txt'
            incoming.write_bytes(sources.assemble(path).replace(b'start_delay=3',b'start_delay=12'))
            with patch.object(transfer,'validate_sequences'),contextlib.redirect_stdout(io.StringIO()):
                plan=transfer.prepare(transfer.Endpoint('local',str(incoming)),transfer.Endpoint('local',str(path)))
                receipt=transfer.commit(plan,root/'backups')
            self.assertTrue(receipt['verified'])
            self.assertEqual(transfer.read(plan.destination),plan.updated)
            self.assertEqual(set(sources.split_bundle(path.read_bytes())),{'universal'})
            self.assertIn(b'start_delay=12',(root/'fighters/kirby.txt').read_bytes())

    def test_packager_assembles_before_mock_compilation_and_packages_exact_bytes(self):
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp);deploy=root/'deployment/ultimate/kirby_assist';deploy.mkdir(parents=True)
            source=fixture(deploy);(deploy/'config.txt').write_text('tech_debug = true\n')
            (root/'Cargo.toml').write_text(f'[package]\nname="test"\nversion="{builder.RELEASE_VERSION}"\ndescription="Test summary"\n')
            expected=sources.assemble(source)
            def compile_mock(*args,**kwargs):
                self.assertEqual((deploy/'assembled/sequences.txt').read_bytes(),expected)
                binary=root/'target/aarch64-skyline-switch/release/libkirby_assist.nro'
                binary.parent.mkdir(parents=True);binary.write_bytes(b'\0'*16+b'NRO0'+b'\0'*100)
            with patch.object(builder,'ROOT',root),patch.object(builder,'documentation_files',return_value={}),patch.object(builder.subprocess,'run',side_effect=compile_mock),patch('sequence_validation.validate_sequences') as validate,contextlib.redirect_stdout(io.StringIO()):
                builder.build_variant('silent',{})
            validate.assert_called_once_with(expected)
            stage=root/'releases/silent'
            self.assertEqual((stage/builder.SEQUENCES).read_bytes(),expected)
            self.assertEqual(source.read_bytes(),UNIVERSAL)


if __name__=='__main__':unittest.main()
