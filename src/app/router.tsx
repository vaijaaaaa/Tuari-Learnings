import React, { useState } from "react";
import {
  Navigate,
  Outlet,
  createBrowserRouter,
  useNavigate,
} from "react-router-dom";
import { AppShell } from "./layout/AppShell";
import { getCurrentRole, getCurrentUser, login } from "../modeles/auth/session";

type Role = "admin" | "user";

function RequireAuthLayout() {
  const role = getCurrentRole();
  if (!role) return <Navigate to="/login" replace />;
  return <AppShell role={role} />;
}

function RoleGuard({ allow }: { allow: Role[] }) {
  const role = getCurrentRole();
  if (!role) return <Navigate to="/login" replace />;
  if (!allow.includes(role)) return <Navigate to="/dashboard" replace />;
  return <Outlet />;
}

function Page({ title }: { title: string }) {
  return (
    <div className="rounded-2xl border border-slate-200 bg-white/80 p-6 shadow-sm">
      <h1 className="text-2xl font-semibold text-slate-900">{title}</h1>
      <p className="mt-2 text-sm text-slate-600">Phase 1 placeholder page</p>
    </div>
  );
}

function LoginPage() {
  const navigate = useNavigate();
  const [username, setUsername] = useState("admin");
  const [password, setPassword] = useState("admin123");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");

  const existing = getCurrentUser();
  if (existing) return <Navigate to="/dashboard" replace />;

  async function onSubmit(e: React.FormEvent<HTMLFormElement>) {
    e.preventDefault();
    setError("");
    setLoading(true);

    try {
      await login(username, password);
      navigate("/dashboard", { replace: true });
    } catch (err) {
      setError(err instanceof Error ? err.message : "Login failed");
    } finally {
      setLoading(false);
    }
  }

  return (
    <div className="min-h-screen p-6">
      <div className="mx-auto mt-16 max-w-md rounded-2xl border border-slate-200 bg-white/90 p-6 shadow-sm">
        <h1 className="text-2xl font-semibold text-slate-900">Sign In</h1>
        <p className="mt-1 text-sm text-slate-600">FramStack Inventory POC</p>

        <form className="mt-6 space-y-4" onSubmit={onSubmit}>
          <div>
            <label className="mb-1 block text-sm text-slate-700">Username</label>
            <input
              className="w-full rounded-xl border border-slate-300 bg-white px-3 py-2 text-sm outline-none focus:border-slate-500"
              value={username}
              onChange={(e) => setUsername(e.currentTarget.value)}
              placeholder="Enter username"
            />
          </div>

          <div>
            <label className="mb-1 block text-sm text-slate-700">Password</label>
            <input
              className="w-full rounded-xl border border-slate-300 bg-white px-3 py-2 text-sm outline-none focus:border-slate-500"
              type="password"
              value={password}
              onChange={(e) => setPassword(e.currentTarget.value)}
              placeholder="Enter password"
            />
          </div>

          {error ? (
            <p className="rounded-lg border border-rose-200 bg-rose-50 px-3 py-2 text-sm text-rose-700">
              {error}
            </p>
          ) : null}

          <button
            type="submit"
            disabled={loading}
            className="w-full rounded-xl bg-slate-900 px-4 py-2 text-sm font-medium text-white transition hover:bg-slate-800 disabled:cursor-not-allowed disabled:opacity-70"
          >
            {loading ? "Signing in..." : "Sign In"}
          </button>
        </form>
      </div>
    </div>
  );
}

export const router = createBrowserRouter([
  { path: "/login", element: <LoginPage /> },
  {
    element: <RequireAuthLayout />,
    children: [
      { path: "/", element: <Navigate to="/dashboard" replace /> },
      { path: "/dashboard", element: <Page title="Dashboard" /> },
      { path: "/inventory", element: <Page title="Inventory" /> },
      { path: "/sales", element: <Page title="Sales" /> },
      { path: "/purchase", element: <Page title="Purchase" /> },
      { path: "/analytics", element: <Page title="Analytics" /> },
      {
        element: <RoleGuard allow={["admin"]} />,
        children: [
          { path: "/admin", element: <Page title="Admin" /> },
          { path: "/users", element: <Page title="Users" /> },
          { path: "/products", element: <Page title="Products" /> },
          { path: "/seasons", element: <Page title="Seasons" /> },
          { path: "/settings", element: <Page title="Settings" /> },
        ],
      },
      { path: "*", element: <Navigate to="/dashboard" replace /> },
    ],
  },
]);