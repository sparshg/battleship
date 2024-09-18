import { io, Socket } from "socket.io-client";

export type Phase = 'placement' | 'battle' | 'gameover';
export type CellType = 'e' | 's' | 'h' | 'm'; // empty, ship, hit, miss

export class State {
    phase: Phase = $state('placement');
    playerBoard = $state(new Board(false));
    opponentBoard = $state(new Board(true));
    room = $state('');
    turn = $state(false);
    socket = io('ws://127.0.0.1:3000/', {
        transports: ['websocket']
    });

    constructor() {
        this.socket.on('created-room', (room: string) => {
            this.room = room;
        });
        this.socket.on('upload', (_, callback) => {
            callback(this.playerBoard.board);
        })
        this.socket.on('turn', (id) => {
            this.turn = id == this.socket.id;
        })
    }

    async attack(i: number, j: number) {
        if (!this.turn) return;
        this.turn = false;
        const res = await this.socket.emitWithAck('attack', [i, j]);
        if (res) {
            this.opponentBoard.board[i][j] = 'h';
        } else {
            this.opponentBoard.board[i][j] = 'm';
        }
    }

    async createRoom() {
        this.socket.emit('create');
        // this.socket.emit('upload', this.playerBoard.board);
        // send the board to the server
        // let api = 'http://127.0.0.1:3000/';
        // await fetch(api, {
        //     method: 'POST',
        //     headers: {
        //         'Content-Type': 'application/json',
        //         'Access-Control-Allow-Origin': '*',
        //     },
        //     body: JSON.stringify(this.playerBoard.board),
        // }).then((response) => {
        //     console.log(response);
        //     response.json().then((data) => {
        //         console.log(data);
        //     });
        // });
    }

    joinRoom() {
        if (this.room.length != 4) return;
        this.socket.emit('join', this.room);
    }
}

export class Board {
    static shipTypes = [5, 4, 3, 3, 2];
    board: Array<Array<CellType>> = $state(Array.from({ length: 10 }, () => Array.from({ length: 10 }, () => 'e')));
    isOpponent: boolean = false;

    constructor(isOpponent: boolean) {
        this.isOpponent = isOpponent;
        if (!isOpponent) this.randomize();
    }

    // set(x: number, y: number, type: CellType) {
    //     this.board[x][y] = type;
    // }

    randomize() {
        this.board = Array.from({ length: 10 }, () => Array.from({ length: 10 }, () => 'e'));
        for (const shipLength of Board.shipTypes) {
            while (true) {
                const dir = Math.round(Math.random());
                const x = Math.floor(Math.random() * (dir ? 10 : 11 - shipLength));
                const y = Math.floor(Math.random() * (dir ? (11 - shipLength) : 10));
                if (this.isOverlapping(x, y, shipLength, dir)) continue;
                for (let i = 0; i < shipLength; i++) {
                    this.board[dir ? x : x + i][dir ? y + i : y] = 's';
                }
                break;
            }
        }
    }

    isOverlapping(x: number, y: number, length: number, dir: number): boolean {
        for (let i = -1; i < 2; i++) {
            for (let j = -1; j < length + 1; j++) {
                let [tx, ty] = [x + (dir ? i : j), y + (dir ? j : i)];
                if (tx < 0 || tx >= 10 || ty < 0 || ty >= 10) continue;
                if (this.board[tx][ty] != 'e') return true;
            }
        }
        return false;
    }

}


