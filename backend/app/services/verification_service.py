from .evidence_service import get_evidence_path
from .hash_service import calculate_sha256


def verify_evidence(
    evidence_id: int,
    expected_hash: str,
) -> bool:
    file_path = get_evidence_path(evidence_id)

    if not file_path.exists():
        return False

    data = file_path.read_bytes()
    actual_hash = calculate_sha256(data)

    return actual_hash == expected_hash
