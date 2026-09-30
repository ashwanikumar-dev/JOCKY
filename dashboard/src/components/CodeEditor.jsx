import React, { useRef, useEffect } from "react";

// JOCKY DSL Syntax Highlighter (Preserving original regex tokenization)
export function highlightJockyCode(code) {
  let text = (code || "")
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;");

  const tokens = [];
  const placeholder = (html) => {
    const id = `___TOK_${tokens.length}___`;
    tokens.push(html);
    return id;
  };

  text = text.replace(/(\/\/.*)/g, (m) => placeholder(`<span class="token-comment">${m}</span>`));
  text = text.replace(/("(\\.|[^"\\])*")/g, (m) => placeholder(`<span class="token-string">${m}</span>`));
  text = text.replace(/\b(\d+(?:KB|MB|GB|TB))\b/g, (m) => placeholder(`<span class="token-bytesize">${m}</span>`));
  text = text.replace(/\b(\d+(?:\.\d+)?)\b/g, (m) => placeholder(`<span class="token-number">${m}</span>`));
  text = text.replace(/\b(true|false)\b/g, (m) => placeholder(`<span class="token-boolean">${m}</span>`));
  text = text.replace(/\b(investigation|collect|from|where|on|every|within|as|to)\b/g, (m) => placeholder(`<span class="token-keyword">${m}</span>`));
  text = text.replace(/\b(System|Process|Network|Evidence)\b/g, (m) => placeholder(`<span class="token-module">${m}</span>`));
  text = text.replace(/\b(info|collect|filter|connections|listeners|preserve|verify)\b/g, (m) => placeholder(`<span class="token-func">${m}</span>`));

  for (let i = 0; i < tokens.length; i++) {
    text = text.replace(`___TOK_${i}___`, tokens[i]);
  }

  return text;
}

export function CodeEditor({ value, onChange, onCompile, editorRef }) {
  const textareaRef = useRef(null);
  const highlightRef = useRef(null);
  const gutterRef = useRef(null);

  // Sync external ref if provided
  useEffect(() => {
    if (editorRef) {
      editorRef.current = textareaRef.current;
    }
  }, [editorRef]);

  const handleScroll = () => {
    if (textareaRef.current) {
      const top = textareaRef.current.scrollTop;
      if (highlightRef.current) highlightRef.current.scrollTop = top;
      if (gutterRef.current) gutterRef.current.scrollTop = top;
    }
  };

  const handleKeyDown = (e) => {
    if (e.key === "Tab") {
      e.preventDefault();
      const textarea = textareaRef.current;
      const start = textarea.selectionStart;
      const end = textarea.selectionEnd;
      const newVal = value.substring(0, start) + "    " + value.substring(end);
      onChange(newVal);
      setTimeout(() => {
        textarea.selectionStart = textarea.selectionEnd = start + 4;
      }, 0);
    } else if (e.key === "Enter" && (e.ctrlKey || e.metaKey)) {
      e.preventDefault();
      onCompile && onCompile();
    }
  };

  const lines = (value || "").split("\n").length;
  const gutterLines = Array.from({ length: lines }, (_, i) => String(i + 1).padStart(2, "0"));

  return (
    <div className="editor-panel">
      <div className="editor-header">
        <div className="editor-file-badge">
          <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
            <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
            <polyline points="14 2 14 8 20 8" />
          </svg>
          <span>endpoint_triage.jocky</span>
        </div>
        <div className="editor-meta-tags">
          <span id="editor-lines-badge" className="pill-label" style={{ fontSize: "10px", padding: "1px 6px" }}>
            {lines} lines
          </span>
          <span className="editor-lang-tag">JOCKY DSL v0.1</span>
        </div>
      </div>

      <div className="editor-body-wrap">
        <div ref={gutterRef} id="editor-gutter" className="editor-gutter">
          {gutterLines.map((num, idx) => (
            <React.Fragment key={idx}>
              {num}
              <br />
            </React.Fragment>
          ))}
        </div>

        <div className="editor-stage-wrap">
          <pre ref={highlightRef} id="editor-highlight" className="editor-highlight">
            <code dangerouslySetInnerHTML={{ __html: highlightJockyCode(value) }} />
          </pre>
          <textarea
            ref={textareaRef}
            id="script-editor"
            spellCheck="false"
            autoComplete="off"
            autoCapitalize="off"
            value={value}
            onChange={(e) => onChange(e.target.value)}
            onScroll={handleScroll}
            onKeyDown={handleKeyDown}
          />
        </div>
      </div>
    </div>
  );
}
