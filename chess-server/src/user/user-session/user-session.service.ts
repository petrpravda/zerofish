import { Injectable } from '@nestjs/common';
import { InjectRepository } from '@nestjs/typeorm';
import { UserSession } from '../entity/user-session.entity';
import { Repository } from 'typeorm';

@Injectable()
export class UserSessionService {
  constructor(
    @InjectRepository(UserSession)
    private userSessionRepository: Repository<UserSession>,
  ) {}

  async create(ipAddress: string, uuid: string): Promise<UserSession> {
    const newUserSession = new UserSession();
    newUserSession.ipAddress = ipAddress;
    newUserSession.uuid = uuid;
    newUserSession.loginTime = new Date();
    newUserSession.lastActive = new Date();
    newUserSession.isConnected = true;
    newUserSession.logoutTime = null;

    await this.userSessionRepository.save(newUserSession);
    return newUserSession;
  }

  // const newUserSession = this.userSessionRepository.create(userSession);
  // await this.userSessionRepository.save(newUserSession);
  // return newUserSession;

  //
  //
  // createUserSession(userId: string, userName: string, ipAddress: string): UserSession {
  //   // Creates a new user session
  // }

  //   getUserSession(sessionId: string): UserSession {
  //     // Retrieves a user session by its ID
  //   }
  //
  //   updateUserSession(sessionId: string, updatedData: Partial<UserSession>): UserSession {
  //     // Updates a user session with new data
  //   }
  //
  //   deleteUserSession(sessionId: string): void {
  //     // Deletes a user session
  //   }
  //
  //   listUserSessions(userId: string): UserSession[] {
  //     // Lists all sessions for a specific user
  //   }
  //
  //   updateGameProgress(sessionId: string, gameProgress: any): UserSession {
  //     // Updates the game progress for a user session
  //   }
  //
  //   updateGameSettings(sessionId: string, gameSettings: any): UserSession {
  //     // Updates the game settings for a user session
  //   }
  //
  //   markUserSessionActive(sessionId: string): UserSession {
  //     // Marks a user session as active
  //   }
  //
  //   markUserSessionInactive(sessionId: string): UserSession {
  //     // Marks a user session as inactive
  //   }
}
