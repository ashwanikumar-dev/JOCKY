// Request validation middleware
module.exports = {
  validateIdParam: (paramName = "id") => {
    return (req, res, next) => {
      const id = req.params[paramName];
      if (!id || typeof id !== "string" || id.trim().length === 0) {
        return res.status(400).json({
          success: false,
          error: {
            status: 400,
            message: `Invalid parameter '${paramName}': cannot be empty.`
          }
        });
      }
      next();
    };
  }
};
