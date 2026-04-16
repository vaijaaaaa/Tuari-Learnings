import { invoke } from "@tauri-apps/api/core";
import { getCurrentRole } from "../auth/session";

export type DashboardSummary = {
  totalProducts: number;
  lowStockCount: number;
  totalStockUnits: number;
  totalSalesAmount: number;
  totalPurchaseAmount: number;
};

function requireRole(): "admin" | "user" {
  const role = getCurrentRole();
  if (!role) throw new Error("Not authenticated");
  return role;
}

export async function fetchDashboardSummary(): Promise<DashboardSummary> {
  const requesterRole = requireRole();
  return invoke<DashboardSummary>("dashboard_summary", { requesterRole });
}
