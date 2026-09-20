from datetime import datetime

from pydantic import BaseModel

from .models import (
    InvestigationStatus,
    TaskStatus,
    EvidenceStatus,
)


class InvestigationCreate(BaseModel):
    script: str


class InvestigationResponse(BaseModel):
    investigation_id: int
    script: str | None
    compiled_ir: list[dict] | None
    status: InvestigationStatus
    created_at: datetime

    model_config = {"from_attributes": True}


class AgentCreate(BaseModel):
    hostname: str
    os: str
    version: str


class AgentResponse(BaseModel):
    agent_id: int
    machine_id: int
    version: str
    last_heartbeat: datetime | None

    model_config = {"from_attributes": True}


class TaskCreate(BaseModel):
    investigation_id: int
    agent_id: int


class TaskResponse(BaseModel):
    task_id: int
    investigation_id: int
    agent_id: int
    ir: list[dict] | None
    status: TaskStatus
    created_at: datetime
    ended_at: datetime | None

    model_config = {"from_attributes": True}


class TaskStatusUpdate(BaseModel):
    status: TaskStatus


class EvidenceCreate(BaseModel):
    task_id: int
    machine_id: int
    agent_id: int

    collector_version: str
    category: str
    raw: str
    normalized: dict | None = None
    collected_at: datetime
    sha256: str


class EvidenceResponse(BaseModel):
    evidence_id: int
    task_id: int
    machine_id: int
    agent_id: int

    collector_version: str
    category: str
    raw: str
    normalized: dict | None
    collected_at: datetime
    sha256: str
    status: EvidenceStatus

    model_config = {"from_attributes": True}
