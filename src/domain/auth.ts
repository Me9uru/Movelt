/** Authenticated account information exposed by the official client. */
export interface LightNovelUser {
  Id: number;
  UserName: string;
  Avatar?: string;
  Email?: string;
  InviteCode?: string;
  UserGroup?: string;
  RegisterAt?: string;
  Growth?: UserGrowth;
}

interface UserGrowth {
  Exp: number;
  Coin: number;
  Level: number;
  GrowthLevel: number;
  CurrentLevelExp: number;
  NextLevelExp?: number;
  SignStreak: number;
  TodaySigned: boolean;
}

/** Credentials used to authenticate an existing account. */
export interface LoginInput {
  email: string;
  password: string;
}

/** Details required to create an account. */
export interface RegisterInput extends LoginInput {
  userName: string;
  code: string;
  inviteCode: string;
}
