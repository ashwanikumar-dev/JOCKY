import React from "react";

export function OsIcon({ osName, size = 18, className = "" }) {
  const os = (osName || "").toLowerCase();

  if (os.includes("windows") || os.includes("win")) {
    return (
      <svg
        className={`os-icon-svg os-windows ${className}`}
        width={size}
        height={size}
        viewBox="0 0 24 24"
        fill="currentColor"
        title="Windows Operating System"
      >
        <path d="M0 3.449L9.75 2.1v9.451H0m10.949-9.602L24 0v11.4H10.949M0 12.6h9.75v9.451L0 20.699M10.949 12.6H24V24l-12.9-1.801" />
      </svg>
    );
  } else if (os.includes("ubuntu") || os.includes("linux") || os.includes("debian")) {
    return (
      <svg
        className={`os-icon-svg os-linux ${className}`}
        width={size}
        height={size}
        viewBox="0 0 24 24"
        fill="none"
        title="Ubuntu / Linux Operating System"
      >
        <circle cx="12" cy="12" r="10" fill="#e95420" />
        <circle cx="12" cy="5.2" r="1.8" fill="#ffffff" />
        <circle cx="6.1" cy="15.4" r="1.8" fill="#ffffff" />
        <circle cx="17.9" cy="15.4" r="1.8" fill="#ffffff" />
        <path
          d="M12 7.2A4.8 4.8 0 0 1 16.8 12c0 .4-.05.8-.15 1.2M7.35 13.2A4.8 4.8 0 0 1 7.2 12a4.8 4.8 0 0 1 3.4-4.6M14.5 15.6A4.8 4.8 0 0 1 12 16.8a4.8 4.8 0 0 1-2.5-.7"
          stroke="#ffffff"
          strokeWidth="1.8"
          strokeLinecap="round"
        />
      </svg>
    );
  }

  return (
    <svg
      className={`os-icon-svg ${className}`}
      width={size}
      height={size}
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="2"
    >
      <rect x="2" y="3" width="20" height="14" rx="2" />
      <path d="M8 21h8M12 17v4" />
    </svg>
  );
}
