from fastapi import APIRouter, Depends, HTTPException
from sqlalchemy.orm import Session
from ..services.compiler_service import compile_script

from .. import models
from ..database import get_db
from ..schemas import InvestigationCreate, InvestigationResponse

router = APIRouter(prefix="/investigations", tags=["Investigations"])


@router.post("", response_model=InvestigationResponse)
def create_investigation(data: InvestigationCreate, db: Session = Depends(get_db)):
    compiled_ir = compile_script(data.script)

    investigation = models.Investigation(
        script=data.script,
        compiled_ir=compiled_ir,
    )

    db.add(investigation)
    db.commit()
    db.refresh(investigation)

    return investigation


@router.get("/{investigation_id}", response_model=InvestigationResponse)
def get_investigation(investigation_id: int, db: Session = Depends(get_db)):
    investigation = db.get(models.Investigation, investigation_id)

    if investigation is None:
        raise HTTPException(status_code=404, detail="Investigation not found")

    return investigation
