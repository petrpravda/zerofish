import { Column, Entity, OneToOne, PrimaryGeneratedColumn } from 'typeorm';
import { User } from './user.entity';

@Entity()
export class UserSession {
  @PrimaryGeneratedColumn()
  id: number;

  @Column()
  uuid: string;

  @Column({ default: () => 'CURRENT_TIMESTAMP' })
  loginTime: Date; // Time when the user logged in

  @Column({ nullable: true })
  logoutTime: Date | null; // Time when the user logged out, null if the user is currently logged in

  @Column({ default: () => 'CURRENT_TIMESTAMP' })
  lastActive: Date; // Last activity time of the user

  @Column()
  ipAddress: string; // IP address of the user

  @Column()
  isConnected: boolean; // Whether the user is currently connected

  @OneToOne(() => User, user => user.session)
  user: User;

  // userId: string; // Unique identifier for the user
  // userName: string; // User's chosen name in the game
  // gameProgress: any; // Object to store game progress, structure depends on the game
  // currentLevel: number; // Current level in the game
  // highestScore: number; // Highest score achieved by the user
  // gameSettings: any; // User's game settings, structure depends on the game
}
