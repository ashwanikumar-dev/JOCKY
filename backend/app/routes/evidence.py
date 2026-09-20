from fastapi import APIRouter, Depends, HTTPException
from sqlalchemy.orm import Session

from .. import models
from ..database import get_db
from ..schemas import EvidenceCreate, EvidenceResponse
from ..services.evidence_service import store_evidence
from ..services.verification_service import verify_evidence

router = APIRouter(prefix="/evidence", tags=["Evidence"])


@router.post("", response_model=EvidenceResponse)
def upload_evidence(
    data: EvidenceCreate,
    db: Session = Depends(get_db),
):
    evidence = models.Evidence(
        task_id=data.task_id,
        machine_id=data.machine_id,
        agent_id=data.agent_id,
        collector_version=data.collector_version,
        category=data.category,
        raw=data.raw,
        normalized=data.normalized,
        collected_at=data.collected_at,
        sha256=data.sha256,
        status=models.EvidenceStatus.COLLECTED,
    )

    db.add(evidence)
    db.flush()

    raw_data = data.raw.encode("utf-8")

    _, file_hash = store_evidence(
        evidence.evidence_id,
        raw_data,
    )

    if file_hash != data.sha256:
        evidence.status = models.EvidenceStatus.INTEGRITY_FAILED

        audit = models.AuditLog(
            evidence_id=evidence.evidence_id,
            actor="backend",
            action="evidence_integrity_failed",
        )

        db.add(audit)
        db.commit()

        raise HTTPException(
            status_code=400,
            detail="Evidence integrity verification failed",
        )

    evidence.status = models.EvidenceStatus.VERIFIED

    audit = models.AuditLog(
        evidence_id=evidence.evidence_id,
        actor="backend",
        action="evidence_verified",
    )

    db.add(audit)
    db.commit()
    db.refresh(evidence)

    return evidence


@router.get("/{evidence_id}/verify")
def verify_evidence_integrity(
    evidence_id: int,
    db: Session = Depends(get_db),
):
    evidence = db.get(models.Evidence, evidence_id)

    if evidence is None:
        raise HTTPException(status_code=404, detail="Evidence not found")

    is_valid = verify_evidence(
        evidence.evidence_id,
        evidence.sha256,
    )

    if not is_valid:
        evidence.status = models.EvidenceStatus.INTEGRITY_FAILED

        audit = models.AuditLog(
            evidence_id=evidence.evidence_id,
            actor="backend",
            action="evidence_integrity_failed",
        )

        db.add(audit)
        db.commit()
    else:
        evidence.status = models.EvidenceStatus.VERIFIED
        db.commit()

    return {
        "evidence_id": evidence.evidence_id,
        "integrity_verified": is_valid,
        "status": evidence.status,
    }


@router.get("/{evidence_id}", response_model=EvidenceResponse)
def get_evidence(
    evidence_id: int,
    db: Session = Depends(get_db),
):
    evidence = db.get(models.Evidence, evidence_id)

    if evidence is None:
        raise HTTPException(status_code=404, detail="Evidence not found")

    return evidence
