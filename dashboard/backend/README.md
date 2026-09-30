# JOCKY Dashboard Backend Service

An architectural API layer built directly inside `dashboard/backend/` conforming to the **JOCKY Engineering Handbook** and **14-Day MVP Implementation & Execution Plan** (SIH Problem Statement 26148, NTRO).

## Architecture

```text
dashboard/
├── src/                 # React + Vite Dashboard
├── public/              # Assets & Brand Logos
├── package.json
└── backend/             # Dashboard Backend Service
    ├── package.json
    ├── .env.example
    ├── README.md
    └── src/
        ├── server.js    # Process bootstrap & lifecycle
        ├── app.js       # Express configuration, CORS, logging
        ├── config/
        │   └── env.js   # Environment variables
        ├── routes/      # Handbook Contract #10 API endpoints
        ├── controllers/ # HTTP Request/Response controllers
        ├── services/    # Business & aggregation logic
        ├── adapters/    # Layer adapters (Compiler, Agent, Evidence, Detection, Timeline)
        ├── repositories/# Data stores (Investigations, Machines, Evidence, Findings, Timeline, Audit)
        ├── middleware/  # Error & 404 handlers, validation
        └── utils/       # Logger
```

## Running the Backend (Windows PowerShell)

```powershell
cd dashboard/backend
npm install
npm run dev
```

Server runs on: `http://localhost:8000`

## API Endpoints (Handbook Contract #10)

- `GET /api/health` - Backend health and service liveness
- `GET /api/dashboard/stats` - Consolidated KPIs and fleet status
- `POST /api/investigations` - Submit script, compile, and create investigation
- `GET /api/investigations` - List all investigations
- `GET /api/investigations/:id` - Fetch single investigation
- `GET /api/machines` - Enrolled fleet endpoints (Windows & Ubuntu)
- `GET /api/machines/:id` - Hostname/ID endpoint telemetry & activity
- `GET /api/evidence` - Cryptographically verified forensic evidence vault
- `GET /api/evidence/:id` - Get specific evidence artifact
- `GET /api/findings` - Security findings and rule detections
- `GET /api/findings/:id` - Get specific finding detail
- `GET /api/timeline` - Chronological cross-machine UTC event stream
- `GET /api/audit` - Chain of custody audit trail
- `POST /api/compile` - JOCKY DSL v0.1 compiler pipeline (AST v1.0 & Forensic IR)
