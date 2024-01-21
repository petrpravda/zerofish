import {
  Controller,
  Get,
  Post,
  Put,
  Delete,
  Param,
  Body, Req
} from '@nestjs/common';
import { UserSessionService } from './user-session.service';
import { UserSession } from '../entity/user-session.entity';
import { Request } from 'express';

@Controller('user-session')
export class UserSessionController {
  constructor(private readonly userSessionService: UserSessionService) {}

  @Post('new')
  async createNew(
    @Req() request: Request,
    @Body() uuid: string,
  ): Promise<UserSession> {
    // Get the IP address from the request
    const ipAddress = request.ip;
    // Call the service method to create a new user session
    return this.userSessionService.create(ipAddress, uuid);
  }

  @Get(':id')
  async findOne(@Param('id') id: string) {
    // Call the service method to find a user session by its ID
  }

  @Put(':id')
  async update(@Param('id') id: string, @Body() userSession: UserSession) {
    // Call the service method to update a user session
  }

  @Delete(':id')
  async remove(@Param('id') id: string) {
    // Call the service method to delete a user session
  }
}
