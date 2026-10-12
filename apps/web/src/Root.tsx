import { lazy, Suspense, useEffect, useState } from "react";
import { Landing } from "./Landing";
const Inspector = lazy(() => import("./App"));
const inspectorRoute = () =>
  location.hash.startsWith("#/inspect") || location.hash === "#inspector";

export default function Root() {
  const [inInspector, setInInspector] = useState(inspectorRoute);
  const [visitedInspector, setVisitedInspector] = useState(inspectorRoute);
  useEffect(() => {
    function navigate() {
      const next = inspectorRoute();
      setInInspector(next);
      if (next) setVisitedInspector(true);
    }
    window.addEventListener("hashchange", navigate);
    return () => window.removeEventListener("hashchange", navigate);
  }, []);
  useEffect(() => {
    document.title = inInspector
      ? "Inspector — Monocle"
      : "Monocle — Finality, in full view";
    if (location.hash === "#/" || location.hash.startsWith("#/inspect"))
      window.scrollTo(0, 0);
  }, [inInspector]);
  return (
    <>
      {!inInspector && <Landing />}
      {visitedInspector && (
        <div hidden={!inInspector}>
          <Suspense
            fallback={
              <main className="route-loading">
                <p role="status">Opening your local inspector…</p>
              </main>
            }
          >
            <Inspector />
          </Suspense>
        </div>
      )}
    </>
  );
}
