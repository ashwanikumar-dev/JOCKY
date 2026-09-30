import React from "react";
import { SearchBar } from "./SearchBar";
import { SystemStatus } from "./SystemStatus";

export function Navbar({ searchValue, onSearchChange, onSearchFocus, nodeCount, isOnline, onLogoClick }) {
  return (
    <header className="topbar">
      <div className="brand" onClick={onLogoClick} style={{ cursor: "pointer" }}>
        <img src="/assets/jocky-logo.png" alt="JOCKY" className="brand-logo-img" />
        <span className="brand-sub">IR Console</span>
      </div>

      <div className="topbar-center">
        <SearchBar value={searchValue} onChange={onSearchChange} onFocus={onSearchFocus} />
      </div>

      <SystemStatus nodeCount={nodeCount} isOnline={isOnline} />
    </header>
  );
}
