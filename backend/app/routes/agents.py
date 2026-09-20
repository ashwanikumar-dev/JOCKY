from datetime import datetime, timezone

from fastapi import APIRouter, Depends
from sqlalchemy.orm import Session

from .. import models
from ..database import get_db
from ..schemas import AgentCreate, AgentResponse

router = APIRouter(prefix="/agents", tags=["Agents"])


@router.post("/register", response_model=AgentResponse)
def register_agent(data: AgentCreate, db: Session = Depends(get_db)):
    machine = models.Machine(hostname=data.hostname, os=data.os, status="online")

    db.add(machine)
    db.flush()

    agent = models.Agent(
        machine_id=machine.machine_id,
        version=data.version,
        last_heartbeat=datetime.now(timezone.utc),
    )

    db.add(agent)
    db.commit()
    db.refresh(agent)

    return agent


@router.get("", response_model=list[AgentResponse])
def get_agents(db: Session = Depends(get_db)):
    return db.query(models.Agent).all()
