import { createFileRoute } from "@tanstack/react-router";

const Editor = () => (
  <main>
    <h1>Editor</h1>
    <p>Coming soon.</p>
  </main>
);

export const Route = createFileRoute("/editor")({ component: Editor });
