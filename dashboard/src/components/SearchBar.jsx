import React, { useEffect, useRef } from "react";

export function SearchBar({ value, onChange, onFocus }) {
  const inputRef = useRef(null);

  useEffect(() => {
    const handleKeyDown = (e) => {
      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "k") {
        e.preventDefault();
        inputRef.current?.focus();
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, []);

  return (
    <div className="search-bar">
      <svg
        className="search-icon"
        width="14"
        height="14"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        strokeWidth="2"
      >
        <circle cx="11" cy="11" r="8" />
        <path d="m21 21-4.35-4.35" />
      </svg>
      <input
        ref={inputRef}
        type="text"
        id="global-search"
        placeholder="Search machines, evidence, findings..."
        value={value || ""}
        onChange={(e) => onChange && onChange(e.target.value)}
        onFocus={onFocus}
      />
      <kbd>Ctrl+K</kbd>
    </div>
  );
}
