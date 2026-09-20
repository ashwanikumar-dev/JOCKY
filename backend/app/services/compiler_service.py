def compile_script(script: str) -> list[dict]:
    return [
        {
            "op": "INVESTIGATION_BEGIN",
            "params": {},
            "investigation_id": 0,
            "ir_version": "0.1",
            "target_scope": "machine",
            "timeout_ms": 30000,
        },
        {
            "op": "INVESTIGATION_END",
            "params": {},
            "investigation_id": 0,
            "ir_version": "0.1",
            "target_scope": "machine",
            "timeout_ms": 30000,
        },
    ]
