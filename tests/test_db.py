from datetime import datetime, timezone

from app.database import SessionLocal
from app.models import Agent, Evidence, Investigation, Machine, Task, AuditLog
from backend.app.services.evidence_service import store_evidence
from backend.app.services.verification_service import verify_evidence

db = SessionLocal()

try:
    # 1. Create machine
    machine = Machine(
        hostname="windows-vm",
        os="Windows",
        status="online",
    )

    db.add(machine)
    db.flush()

    # 2. Create agent for this machine
    agent = Agent(
        machine_id=machine.machine_id,
        version="1.0.0",
        last_heartbeat=datetime.now(timezone.utc),
    )

    db.add(agent)

    # 3. Create investigation
    investigation = Investigation(
        script="processes where memory > 80%",
    )

    db.add(investigation)
    db.flush()

    # 4. Create task
    task = Task(
        investigation_id=investigation.investigation_id,
        agent_id=agent.agent_id,
    )

    db.add(task)
    db.flush()

    raw_evidence = b"""
{
    "process_name": "chrome.exe",
    "pid": 1234,
    "memory_percent": 92.5
}
"""

    evidence = Evidence(
        investigation_id=investigation.investigation_id,
        task_id=task.task_id,
        machine_id=machine.machine_id,
        agent_id=agent.agent_id,
        hostname=machine.hostname,
        operating_system=machine.os,
        evidence_type="process",
        collector="process_collector",
        collector_version="1.0.0",
        collection_time=datetime.now(timezone.utc),
        normalized_data={
            "process_name": "chrome.exe",
            "pid": 1234,
            "memory_percent": 92.5,
        },
        evidence_metadata={
            "source": "windows-vm",
            "collector_status": "success",
        },
        hash="dummy_hash_for_now",
        storage_reference="storage/evidence/test-evidence.json",
    )

    db.add(evidence)
    db.flush()

    storage_reference, file_hash = store_evidence(
        evidence.evidence_id,
        raw_evidence,
    )
    evidence.storage_reference = storage_reference
    evidence.hash = file_hash

    is_valid = verify_evidence(
        evidence.storage_reference,
        evidence.hash,
    )

    audit = AuditLog(
        evidence_id=evidence.evidence_id,
        actor="backend",
        action="evidence_verified",
    )

    db.add(audit)
    db.commit()

    print(
        "Audit:",
        audit.actor,
        audit.action,
    )

    print("Data created successfully!")
    print("Machine:", machine.machine_id)
    print("Agent:", agent.agent_id)
    print("Investigation:", investigation.investigation_id)
    print("Task:", task.task_id)

    print("\nEvidence:")
    print("Evidence ID:", evidence.evidence_id)
    print("Type:", evidence.evidence_type)
    print("Collector:", evidence.collector)
    print("Process:", evidence.normalized_data["process_name"])
    print("Memory:", evidence.normalized_data["memory_percent"])

    print("\nEvidence integrity:")
    print("Storage:", evidence.storage_reference)
    print("SHA-256:", evidence.hash)
    print("Integrity verified:", is_valid)

    print("\nRelationship check:")

    print("Task belongs to investigation:", task.investigation.investigation_id)

    print("Task assigned to agent:", task.agent.agent_id)

    print("Agent belongs to machine:", task.agent.machine.hostname)


finally:
    db.close()
