import { io, Socket } from "socket.io-client";

export type Phase = 'placement' | 'waiting' | 'selfturn' | 'otherturn';
export type CellType = 'e' | 's' | 'h' | 'm'; // empty, ship, hit, miss

export class State {
    phase: Phase = $state('placement');
    playerBoard = $state(new Board(false));
    opponentBoard = $state(new Board(true));
    users = $state(0);
    room = $state('');
    turn = $state(-1); // -1 not my turn, 0 might be, 1 is
    socket: Socket;

    constructor(hostname: string) {
        // let session = sessionStorage.getItem('session');

        this.socket = io(`ws://${hostname}:3000/`, {
            transports: ['websocket'],
            auth: { session: sessionStorage.getItem('session') }
        });

        this.socket.on('connect', () => {
            console.log(this.socket.id);
            sessionStorage.setItem('session', this.socket.id!);
        });

        this.socket.on('update-room', ({ room, users }) => {
            if (this.phase == 'placement') this.phase = 'waiting';
            this.room = room;
            this.users = users;
        });

        this.socket.on('upload', (_, callback) => {
            callback(this.playerBoard.board);
        });
        this.socket.on('turnover', (id) => {
            this.turn = (id == this.socket.id) ? 1 : -1;
            this.phase = this.turn ? 'selfturn' : 'otherturn';
        });
        this.socket.on('attacked', ({ by, at, hit, sunk }) => {
            const [i, j]: [number, number] = at;
            let board = by == this.socket.id ? this.opponentBoard : this.playerBoard;
            if (by == this.socket.id) {
                this.turn = (hit) ? 1 : -1;
            } else {
                this.turn = (!hit) ? 1 : -1;
            }
            if (hit) {
                board.board[i][j] = 'h';
                for (let [x, y] of [[-1, -1], [1, 1], [1, -1], [-1, 1]]) {
                    const [tx, ty] = [i + x, j + y];
                    if (tx < 0 || tx >= 10 || ty < 0 || ty >= 10) continue;
                    if (board.board[tx][ty] == 'e')
                        board.board[tx][ty] = 'm';
                }
            } else {
                board.board[i][j] = 'm';
            }
            if (sunk) {
                const [[minx, miny], [maxx, maxy]] = sunk;
                const x1 = Math.max(0, minx - 1);
                const y1 = Math.max(0, miny - 1);
                const x2 = Math.min(9, maxx + 1);
                const y2 = Math.min(9, maxy + 1);
                for (let x = x1; x <= x2; x++) {
                    for (let y = y1; y <= y2; y++) {
                        if (board.board[x][y] == 'e') {
                            board.board[x][y] = 'm';
                        }
                    }
                }
            }
        });

        this.socket.on('restore', ({ turn, player, opponent }: { turn: boolean, player: string[], opponent: string[] }) => {
            this.turn = turn ? 1 : -1;
            this.phase = this.turn ? 'selfturn' : 'otherturn';
            this.playerBoard.board = player.map((s) => s.split('').map(c => c as CellType));
            this.opponentBoard.board = opponent.map((s) => s.split('').map(c => c as CellType));
        })
    }

    attack(i: number, j: number) {
        if (this.turn != 1) return;
        if (this.opponentBoard.board[i][j] != 'e') return;
        this.turn = 0;

        this.socket.emit('attack', [i, j]);
    }

    createRoom() {
        this.socket.emit('create');
    }

    joinRoom(code: string) {
        code = code.toUpperCase();
        if (code.length != 4 || code == this.room) return;
        this.socket.emit('join', code);
    }

    hasNotStarted() {
        return this.phase == 'placement' || this.phase == 'waiting';
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


