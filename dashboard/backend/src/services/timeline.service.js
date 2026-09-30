// Timeline Service
const timelineAdapter = require("../adapters/timeline.adapter");

module.exports = {
  getAll: async (investigationId) => {
    return await timelineAdapter.getTimelineStream(investigationId);
  },

  create: async (data) => {
    return await timelineAdapter.recordTimelineEvent(data);
  }
};
