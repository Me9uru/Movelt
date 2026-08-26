/** Authenticated account information exposed by the official client. */
export interface LightNovelUser {
  Id: number;
  UserName: string;
  Avatar?: string;
  Email?: string;
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
