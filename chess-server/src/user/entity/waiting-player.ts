import { Entity, JoinColumn, OneToOne, PrimaryGeneratedColumn } from 'typeorm';
import { User } from './user.entity';

@Entity()
export class WaitingPlayer {
  @PrimaryGeneratedColumn()
  id: number;

  @OneToOne(() => User, user => user.waitingForGame)
  @JoinColumn()
  user: User;
}
