import React from "react";
import { Navigate, Outlet, createBrowserRouter } from "react-router-dom";
import { AppShell } from "./layout/AppShell";

type Role = "admin" | "user";

// Temporary mock role for Phase 0. Replace with real auth/session in Phase 1.
const currentRole: Role = "admin";

function RoleGuard({ allow }: { allow: Role[] }) {
  if (!allow.includes(currentRole)) {
    return <Navigate to="/dashboard" replace />;
  }

  return <Outlet />;
}

function Page({ title }: { title: string }) {
  return (
    <div className="rounded-2xl border border-slate-200 bg-white/80 p-6 shadow-sm">
      <h1 className="text-2xl font-semibold text-slate-900">{title}</h1>
      <p className="mt-2 text-sm text-slate-600">Phase 0 placeholder page</p>
    </div>
  );
}

export const router = createBrowserRouter([
  {
    element: <AppShell role={currentRole} />,
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