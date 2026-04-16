import { invoke } from "@tauri-apps/api/core";
import { getCurrentRole, getCurrentUser } from "../auth/session";

export type InventoryItem = {
  productId: number;
  sku: string;
  name: string;
  quantity: number;
  updatedAt: string | null;
};

export type AdjustStockInput = {
  productId: number;
  quantityDelta: number;
  note?: string;
};

function requireRole(): "admin" | "user" {
  const role = getCurrentRole();
  if (!role) throw new Error("Not authenticated");
  return role;
}

function requireUsername(): string {
  const user = getCurrentUser();
  if (!user) throw new Error("Not authenticated");
  return user.username;
}

export async function fetchInventory(): Promise<InventoryItem[]> {
  const requesterRole = requireRole();
  return invoke<InventoryItem[]>("list_inventory", { requesterRole });
}

export async function adjustStock(input: AdjustStockInput): Promise<InventoryItem> {
  const requesterRole = requireRole();
  const requesterUsername = requireUsername();

  return invoke<InventoryItem>("adjust_stock", {
    requesterRole,
    requesterUsername,
    input,
  });
}
