import { invoke } from "@tauri-apps/api/core";
import { getCurrentRole, getCurrentUser } from "../auth/session";

export type PurchaseListItem = {
  id: number;
  totalAmount: number;
  note: string | null;
  createdBy: string | null;
  itemCount: number;
  createdAt: string;
};

export type PurchaseItemInput = {
  productId: number;
  quantity: number;
  unitCost: number;
};

export type CreatePurchaseInput = {
  items: PurchaseItemInput[];
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

export async function fetchPurchases(): Promise<PurchaseListItem[]> {
  const requesterRole = requireRole();
  return invoke<PurchaseListItem[]>("list_purchases", { requesterRole });
}

export async function createPurchase(input: CreatePurchaseInput) {
  const requesterRole = requireRole();
  const requesterUsername = requireUsername();

  return invoke("create_purchase", {
    requesterRole,
    requesterUsername,
    input,
  });
}
