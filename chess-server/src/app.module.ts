import { Module } from '@nestjs/common';
import { AppController } from './app.controller';
import { AppService } from './app.service';
import { ServeStaticModule } from '@nestjs/serve-static';
import { join } from 'path';
import { TypeOrmModule } from '@nestjs/typeorm';
import { UserModule } from './user/user.module';
import { SnakeNamingStrategy } from 'typeorm-naming-strategies';

const rootPath = join(__dirname, '..', 'static');
const databaseUrl = process.env.DATABASE_URL;

@Module({
  imports: [
    ServeStaticModule.forRoot({
      rootPath: rootPath,
    }),
    TypeOrmModule.forRoot({
      type: 'postgres',
      url: databaseUrl,
      autoLoadEntities: true,
      synchronize: true,
      namingStrategy: new SnakeNamingStrategy(),
    }),
    UserModule,
  ],
  controllers: [AppController],
  providers: [AppService],
})
export class AppModule {}
