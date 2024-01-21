import { Module } from '@nestjs/common';
import { TypeOrmModule } from '@nestjs/typeorm';
import { UserSession } from './entity/user-session.entity';
import { UserSessionService } from './user-session/user-session.service';
import { UserSessionController } from './user-session/user-session.controller';

@Module({
  imports: [TypeOrmModule.forFeature([UserSession])],
  providers: [UserSessionService],
  controllers: [UserSessionController],
})
export class UserModule {}
