import { Column, Entity, JoinColumn, OneToOne, PrimaryGeneratedColumn } from 'typeorm';
import { User } from './user.entity';

@Entity()
export class ActiveGame {
  @PrimaryGeneratedColumn()
  id: number;

  @Column({ length: 100, nullable: true })
  playerWhiteName: string;

  @Column({ length: 100, nullable: true })
  playerBlackName: string;

  @OneToOne(() => User, user => user.activeGame, {nullable: true})
  @JoinColumn()
  playerWhite: User | null;

  @OneToOne(() => User, user => user.activeGame, {nullable: true})
  @JoinColumn()
  playerBlack: User | null;

  @Column('text')
  pgnMoves: string;
}
