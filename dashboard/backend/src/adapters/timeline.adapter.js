// JOCKY Timeline Engine Adapter
// Interface: Layer 5 (Forensic Intelligence) -> Layer 6 (Platform)
// Adheres to Handbook Contract #9 and Section 27 (Cross-machine UTC Normalized Timeline)

const timelineRepo = require("../repositories/timeline.repository");

module.exports = {
  /**
   * Retrieve chronologically ordered cross-machine event stream
   * @param {string} [investigationId]
   */
  getTimelineStream: async (investigationId) => {
    return await timelineRepo.findAll(investigationId);
  },

  /**
   * Append new timeline event
   * @param {object} eventData
   */
  recordTimelineEvent: async (eventData) => {
    return await timelineRepo.create(eventData);
  }
};
