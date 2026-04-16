import { invoke } from "@tauri-apps/api/core";
import { getCurrentRole } from "../../auth/session";

export type SeasonListItem = {
  id: number;
  name: string;
  startDate: string | null;
  endDate: string | null;
  isActive: boolean;
  createdAt: string;
};

export type CreateSeasonInput = {
  name: string;
  startDate?: string;
  endDate?: string;
};

function requireRole(): "admin" | "user" {
  const role = getCurrentRole();
  if (!role) {
    throw new Error("Not authenticated");
  }
  return role;
}

export async function fetchSeasons(): Promise<SeasonListItem[]> {
  const requesterRole = requireRole();

  const seasons = await invoke<SeasonListItem[]>("list_seasons", {
    requesterRole,
  });

  return seasons;
}

export async function createSeason(input: CreateSeasonInput): Promise<SeasonListItem> {
  const requesterRole = requireRole();

  const created = await invoke<SeasonListItem>("create_season", {
    requesterRole,
    input,
  });

  return created;
}
