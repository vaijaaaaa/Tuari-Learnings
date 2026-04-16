import { NavLink, Outlet } from "react-router-dom";

type Role = "admin" | "user";

type NavItem = {
  to: string;
  label: string;
};

const commonItems: NavItem[] = [
  { to: "/dashboard", label: "Dashboard" },
  { to: "/inventory", label: "Inventory" },
  { to: "/sales", label: "Sales" },
  { to: "/purchase", label: "Purchase" },
  { to: "/analytics", label: "Analytics" },
];

const adminItems: NavItem[] = [
  { to: "/admin", label: "Admin" },
  { to: "/users", label: "Users" },
  { to: "/products", label: "Products" },
  { to: "/seasons", label: "Seasons" },
  { to: "/settings", label: "Settings" },
];

function ItemLink({ to, label }: NavItem) {
  return (
    <NavLink
      to={to}
      className={({ isActive }) =>
        [
          "block rounded-xl px-3 py-2 text-sm transition",
          isActive
            ? "bg-slate-900 text-white"
            : "text-slate-700 hover:bg-slate-200/70",
        ].join(" ")
      }
    >
      {label}
    </NavLink>
  );
}

export function AppShell({ role }: { role: Role }) {
  const items = role === "admin" ? [...commonItems, ...adminItems] : commonItems;

  return (
    <div className="min-h-screen p-4 md:p-6">
      <div className="mx-auto grid max-w-7xl grid-cols-1 gap-4 md:grid-cols-[240px_1fr]">
        <aside className="rounded-2xl border border-slate-200 bg-white/80 p-4 shadow-sm">
          <div className="mb-4">
            <p className="text-xs uppercase tracking-wide text-slate-500">FramStack</p>
            <h1 className="text-lg font-semibold text-slate-900">Inventory POC</h1>
          </div>

          <nav className="space-y-1">
            {items.map((item) => (
              <ItemLink key={item.to} to={item.to} label={item.label} />
            ))}
          </nav>
        </aside>

        <div className="space-y-4">
          <header className="flex items-center justify-between rounded-2xl border border-slate-200 bg-white/80 px-4 py-3 shadow-sm">
            <div>
              <p className="text-xs uppercase tracking-wide text-slate-500">Environment</p>
              <p className="text-sm font-medium text-slate-900">POC Desktop</p>
            </div>
            <div className="rounded-full border border-slate-300 bg-slate-50 px-3 py-1 text-xs font-medium text-slate-700">
              Role: {role}
            </div>
          </header>

          <main>
            <Outlet />
          </main>
        </div>
      </div>
    </div>
  );
}