import { Module } from '@nestjs/common';
import { TypeOrmModule } from '@nestjs/typeorm';
import { UserSession } from './entity/user-session.entity';
import { UserSessionService } from './user-session/user-session.service';
import { UserSessionController } from './user-session/user-session.controller';
import { User } from './entity/user.entity';
import { Game } from './entity/game.entity';
import { ActiveGame } from './entity/active-game.entity';
import { WaitingPlayer } from './entity/waiting-player';

@Module({
  imports: [
    TypeOrmModule.forFeature([
      UserSession,
      User,
      Game,
      ActiveGame,
      WaitingPlayer,
    ]),
  ],
  providers: [UserSessionService],
  controllers: [UserSessionController],
})
export class UserModule {}
