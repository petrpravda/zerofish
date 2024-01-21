import { Test, TestingModule } from '@nestjs/testing';
import { UserSessionController } from './user-session.controller';

describe('UserSessionController', () => {
  let controller: UserSessionController;

  beforeEach(async () => {
    const module: TestingModule = await Test.createTestingModule({
      controllers: [UserSessionController],
    }).compile();

    controller = module.get<UserSessionController>(UserSessionController);
  });

  it('should be defined', () => {
    expect(controller).toBeDefined();
  });
});
