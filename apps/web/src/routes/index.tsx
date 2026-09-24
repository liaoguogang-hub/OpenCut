import { createFileRoute } from "@tanstack/react-router";

const Home = () => (
  <div>
    <p>hello world!</p>
  </div>
);

export const Route = createFileRoute("/")({ component: Home });
