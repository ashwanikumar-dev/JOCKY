from pathlib import Path

from .hash_service import calculate_sha256

EVIDENCE_DIR = Path("storage/evidence")


def get_evidence_path(evidence_id: int) -> Path:
    return EVIDENCE_DIR / f"{evidence_id}.json"


def store_evidence(evidence_id: int, data: bytes) -> tuple[str, str]:
    EVIDENCE_DIR.mkdir(parents=True, exist_ok=True)

    file_path = get_evidence_path(evidence_id)
    file_path.write_bytes(data)

    file_hash = calculate_sha256(data)

    return str(file_path), file_hash
