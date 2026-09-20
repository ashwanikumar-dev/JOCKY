from fastapi import APIRouter, Depends
from sqlalchemy.orm import Session

from .. import models
from ..database import get_db

router = APIRouter(prefix="/audit", tags=["Audit"])


@router.get("")
def get_audit_logs(
    investigation_id: int,
    db: Session = Depends(get_db),
):
    logs = (
        db.query(models.AuditLog)
        .join(models.Evidence)
        .join(models.Task)
        .filter(models.Task.investigation_id == investigation_id)
        .all()
    )

    return logs
