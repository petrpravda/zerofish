import { Module } from '@nestjs/common';
import { AppController } from './app.controller';
import { AppService } from './app.service';
import { ServeStaticModule } from '@nestjs/serve-static';
import { join } from 'path';

const rootPath = join(__dirname, '..', 'static');
console.info(rootPath);

@Module({
  imports: [
    ServeStaticModule.forRoot({
      rootPath: rootPath,
    }),
  ],
  controllers: [AppController],
  providers: [AppService],
})
export class AppModule {}
