import { Entity, PrimaryGeneratedColumn, Column, ManyToOne } from 'typeorm';
import { User } from './user.entity';

@Entity()
export class Game {
  @PrimaryGeneratedColumn()
  id: number;

  @Column({ length: 100, nullable: true })
  playerWhiteName: string;

  @Column({ length: 100, nullable: true })
  playerBlackName: string;

  @ManyToOne(() => User, (user) => user.whiteGames)
  playerWhite: User;

  @ManyToOne(() => User, (user) => user.blackGames)
  playerBlack: User;

  @Column('text')
  pgnMoves: string;

  // @Column({ nullable: true })
  // result: string;
  //
  // @Column({ type: 'timestamp', default: () => 'CURRENT_TIMESTAMP' })
  // gameDate: Date;
  //
  // @Column({ default: false })
  // isDraw: boolean;
  //
  // @Column({ default: false })
  // isCheckmate: boolean;
  //
  // @Column({ default: false })
  // isStalemate: boolean;
  //
  // @Column({ default: false })
  // isThreefoldRepetition: boolean;
  //
  // @Column({ default: false })
  // isInsufficientMaterial: boolean;
  //
  // @Column({ default: false })
  // isSeventyFiveMoveRule: boolean;
  //
  // @Column({ default: false })
  // isFivefoldRepetition: boolean;
  //
  // @Column({ nullable: true })
  // opening: string;
  //
  // @Column({ nullable: true })
  // eco: string;
  //
  // @Column({ nullable: true })
  // termination: string;
}
