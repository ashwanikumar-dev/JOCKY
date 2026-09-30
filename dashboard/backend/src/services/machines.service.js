// Machines Service
// Interfaces with agent adapter and machines repository

const agentAdapter = require("../adapters/agent.adapter");

module.exports = {
  getAll: async () => {
    return await agentAdapter.getRegisteredAgents();
  },

  getById: async (id) => {
    return await agentAdapter.getAgentById(id);
  },

  register: async (data) => {
    return await agentAdapter.registerAgent(data);
  }
};
