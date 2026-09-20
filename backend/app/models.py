from sqlalchemy import (
    Integer,
    String,
    Text,
    DateTime,
    ForeignKey,
    JSON,
    Enum as SQLEnum,
)
from sqlalchemy.orm import Mapped, mapped_column, relationship

from .database import Base

from datetime import datetime, timezone
from enum import Enum


class InvestigationStatus(str, Enum):
    PENDING = "pending"
    RUNNING = "running"
    COMPLETED = "completed"
    FAILED = "failed"


class TaskStatus(str, Enum):
    CREATED = "created"
    DISPATCHED = "dispatched"
    RUNNING = "running"
    COMPLETED = "completed"
    FAILED = "failed"


class EvidenceStatus(str, Enum):
    COLLECTED = "collected"
    UPLOADED = "uploaded"
    VERIFIED = "verified"
    INTEGRITY_FAILED = "integrity_failed"


class Investigation(Base):
    __tablename__ = "investigations"

    investigation_id: Mapped[int] = mapped_column(Integer, primary_key=True)

    script: Mapped[str | None] = mapped_column(Text, nullable=True)

    compiled_ir: Mapped[dict | None] = mapped_column(JSON, nullable=True)

    status: Mapped[InvestigationStatus] = mapped_column(
        SQLEnum(InvestigationStatus), default=InvestigationStatus.PENDING
    )

    created_at: Mapped[datetime] = mapped_column(
        DateTime(timezone=True), default=lambda: datetime.now(timezone.utc)
    )

    tasks: Mapped[list["Task"]] = relationship(back_populates="investigation")


class Machine(Base):
    __tablename__ = "machines"

    machine_id: Mapped[int] = mapped_column(Integer, primary_key=True)

    hostname: Mapped[str] = mapped_column(String(255))

    os: Mapped[str] = mapped_column(String(100))

    status: Mapped[str] = mapped_column(String(50), default="offline")

    agent: Mapped["Agent | None"] = relationship(
        back_populates="machine", uselist=False
    )


class Agent(Base):
    __tablename__ = "agents"

    agent_id: Mapped[int] = mapped_column(Integer, primary_key=True)

    machine_id: Mapped[int] = mapped_column(
        ForeignKey("machines.machine_id"), unique=True
    )

    version: Mapped[str] = mapped_column(String(50))

    last_heartbeat: Mapped[datetime | None] = mapped_column(
        DateTime(timezone=True), nullable=True
    )

    machine: Mapped["Machine"] = relationship(back_populates="agent")

    tasks: Mapped[list["Task"]] = relationship(back_populates="agent")


class Task(Base):
    __tablename__ = "tasks"

    task_id: Mapped[int] = mapped_column(Integer, primary_key=True)

    investigation_id: Mapped[int] = mapped_column(
        ForeignKey("investigations.investigation_id")
    )

    agent_id: Mapped[int] = mapped_column(ForeignKey("agents.agent_id"))

    ir: Mapped[list | None] = mapped_column(JSON, nullable=True)

    status: Mapped[TaskStatus] = mapped_column(
        SQLEnum(TaskStatus), default=TaskStatus.CREATED
    )

    created_at: Mapped[datetime] = mapped_column(
        DateTime(timezone=True), default=lambda: datetime.now(timezone.utc)
    )

    ended_at: Mapped[datetime | None] = mapped_column(
        DateTime(timezone=True), nullable=True
    )

    investigation: Mapped["Investigation"] = relationship(back_populates="tasks")

    agent: Mapped["Agent"] = relationship(back_populates="tasks")

    evidence: Mapped[list["Evidence"]] = relationship(back_populates="task")


class Evidence(Base):
    __tablename__ = "evidence"

    evidence_id: Mapped[int] = mapped_column(Integer, primary_key=True)

    task_id: Mapped[int] = mapped_column(ForeignKey("tasks.task_id"))

    machine_id: Mapped[int] = mapped_column(ForeignKey("machines.machine_id"))

    agent_id: Mapped[int] = mapped_column(ForeignKey("agents.agent_id"))

    collector_version: Mapped[str] = mapped_column(String(50))

    category: Mapped[str] = mapped_column(String(100))

    raw: Mapped[str] = mapped_column(Text)

    normalized: Mapped[dict | None] = mapped_column(JSON, nullable=True)

    collected_at: Mapped[datetime] = mapped_column(DateTime(timezone=True))

    sha256: Mapped[str] = mapped_column(String(64))

    status: Mapped[EvidenceStatus] = mapped_column(
        SQLEnum(EvidenceStatus), default=EvidenceStatus.COLLECTED
    )

    task: Mapped["Task"] = relationship(back_populates="evidence")

    findings: Mapped[list["Finding"]] = relationship(back_populates="evidence")

    audit_logs: Mapped[list["AuditLog"]] = relationship(back_populates="evidence")

    timeline_events: Mapped[list["TimelineEvent"]] = relationship(
        back_populates="evidence"
    )


class Finding(Base):
    __tablename__ = "findings"

    finding_id: Mapped[int] = mapped_column(Integer, primary_key=True)

    evidence_id: Mapped[int] = mapped_column(ForeignKey("evidence.evidence_id"))

    rule_id: Mapped[str] = mapped_column(String(100))

    severity: Mapped[str] = mapped_column(String(50))
    reason: Mapped[str] = mapped_column(Text)

    evidence: Mapped["Evidence"] = relationship(back_populates="findings")

    alerts: Mapped[list["Alert"]] = relationship(back_populates="finding")


class Alert(Base):
    __tablename__ = "alerts"

    alert_id: Mapped[int] = mapped_column(Integer, primary_key=True)

    finding_id: Mapped[int] = mapped_column(ForeignKey("findings.finding_id"))

    risk_score: Mapped[int] = mapped_column(Integer)

    finding: Mapped["Finding"] = relationship(back_populates="alerts")


class AuditLog(Base):
    __tablename__ = "audit_logs"

    audit_id: Mapped[int] = mapped_column(Integer, primary_key=True)

    evidence_id: Mapped[int] = mapped_column(ForeignKey("evidence.evidence_id"))

    actor: Mapped[str] = mapped_column(String(100))

    action: Mapped[str] = mapped_column(String(100))

    timestamp: Mapped[datetime] = mapped_column(
        DateTime(timezone=True), default=lambda: datetime.now(timezone.utc)
    )

    evidence: Mapped["Evidence"] = relationship(back_populates="audit_logs")


class TimelineEvent(Base):
    __tablename__ = "timeline_events"

    timeline_id: Mapped[int] = mapped_column(Integer, primary_key=True)

    evidence_id: Mapped[int] = mapped_column(ForeignKey("evidence.evidence_id"))

    timestamp_utc: Mapped[datetime] = mapped_column(DateTime(timezone=True))

    evidence: Mapped["Evidence"] = relationship(back_populates="timeline_events")
