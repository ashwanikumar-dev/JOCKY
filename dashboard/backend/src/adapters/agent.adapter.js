// JOCKY Agent & Fleet Adapter
// Interface: Layer 4 (OS Integration / Endpoints) -> Layer 6 (Platform)
// Adheres to Handbook Contract #5 (Agent Task Contract) and Contract #8 (Agent -> Backend API)

const machinesRepo = require("../repositories/machines.repository");

module.exports = {
  /**
   * Fetch all registered endpoint agents in the fleet
   */
  getRegisteredAgents: async () => {
    return await machinesRepo.findAll();
  },

  /**
   * Fetch a specific agent node by machine_id or hostname
   */
  getAgentById: async (id) => {
    return await machinesRepo.findById(id);
  },

  /**
   * Register a new agent or heartbeat check
   */
  registerAgent: async (agentData) => {
    return await machinesRepo.create(agentData);
  }
};
