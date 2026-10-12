export function Mark({ className = "" }: { className?: string }) {
  return (
    <svg
      className={className}
      width="32"
      height="32"
      viewBox="0 0 32 32"
      fill="none"
      aria-hidden="true"
    >
      <circle
        cx="14"
        cy="14"
        r="10.5"
        stroke="currentColor"
        strokeWidth="2.1"
      />
      <circle cx="14" cy="14" r="5" stroke="currentColor" strokeWidth="1.4" />
      <path
        d="M22 22v6h6"
        stroke="currentColor"
        strokeWidth="2.1"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
      <circle cx="14" cy="14" r="1.5" fill="currentColor" />
    </svg>
  );
}
export function Brand({ className = "" }: { className?: string }) {
  return (
    <span className={`monocle-brand ${className}`}>
      <Mark />
      <span>
        monocle<span className="monocle-brand-dot">.</span>
      </span>
    </span>
  );
}
export function Arrow({ diagonal = false }: { diagonal?: boolean }) {
  return (
    <svg
      width="20"
      height="20"
      viewBox="0 0 24 24"
      fill="none"
      aria-hidden="true"
    >
      <path
        d={diagonal ? "M6 18 18 6M6 6h12v12" : "M4 12h15m-6-6 6 6-6 6"}
        stroke="currentColor"
        strokeWidth="1.6"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
    </svg>
  );
}
