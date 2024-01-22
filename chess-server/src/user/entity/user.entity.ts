import { Column, Entity, JoinColumn, OneToMany, OneToOne, PrimaryGeneratedColumn } from 'typeorm';
import { UserSession } from './user-session.entity';
import { Game } from './game.entity';

@Entity()
export class User {
  @PrimaryGeneratedColumn()
  id: number;

  @Column({ unique: true })
  username: string;

  @Column()
  password: string; // hash

  @Column({ nullable: true })
  email: string;

  @Column({ default: 0 })
  eloRating: number;

  @Column({ default: () => 'CURRENT_TIMESTAMP' })
  createdAt: Date;

  @Column({ default: () => 'CURRENT_TIMESTAMP' })
  updatedAt: Date;

  @OneToOne(() => UserSession, (userSession) => userSession.user, {
    eager: true,
  })
  @JoinColumn()
  session: UserSession;

  @OneToMany(() => Game, (game) => game.playerWhite)
  whiteGames: Game[];

  @OneToMany(() => Game, (game) => game.playerBlack)
  blackGames: Game[];
  // historical games
}
