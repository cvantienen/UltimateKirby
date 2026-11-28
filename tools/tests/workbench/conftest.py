import pytest
import sequence_sources as sources
import workbench


@pytest.fixture(autouse=True)
def isolate_repository_sequence_sources(tmp_path, monkeypatch):
    """Prevent CLI import tests from modifying the developer checkout sources."""
    sequences = tmp_path / "deployment" / "ultimate" / "kirby_assist" / "sequences.txt"
    sequences.parent.mkdir(parents=True)
    sequences.write_text("[universal.settings]\nenabled = true\n")
    (sequences.parent / "fighters").mkdir()
    monkeypatch.setattr(sources, "ROOT", tmp_path)
    monkeypatch.setattr(sources, "SOURCE", sequences)
    monkeypatch.setattr(sources, "GENERATED", sequences.parent / "assembled" / "sequences.txt")
    monkeypatch.setattr(workbench.sources, "ROOT", tmp_path)
    monkeypatch.setattr(workbench.sources, "SOURCE", sequences)
    monkeypatch.setattr(workbench.sources, "GENERATED", sequences.parent / "assembled" / "sequences.txt")
    monkeypatch.setattr(workbench, "ROOT", tmp_path)
    monkeypatch.setattr(workbench, "PROJECT_ROOT", tmp_path)
    monkeypatch.setattr(workbench, "DEFAULT_FILE", sequences.parent / "assembled" / "sequences.txt")
    monkeypatch.setattr(workbench.workbench_history, "ROOT", tmp_path)
    monkeypatch.setattr(workbench.upload_switch, "PROJECT_ROOT", tmp_path)
    monkeypatch.setattr(workbench.upload_switch, "DEFAULT_RELEASE_ROOT", tmp_path.parent)
    monkeypatch.setattr(workbench.upload_switch, "DEPLOYMENT_SEQUENCES", sequences)
