export type Phase = 'placement' | 'battle' | 'gameover';
export type CellType = 'e' | 's' | 'h' | 'm'; // empty, ship, hit, miss

export class State {
    phase: Phase = $state('placement');
    playerBoard = $state(new Board(false));
    opponentBoard = $state(new Board(true));
}

export class Board {
    static shipTypes = [5, 4, 3, 3, 2];
    board: Array<Array<CellType>> = $state(Array.from({ length: 10 }, () => Array.from({ length: 10 }, () => 'e')));
    isOpponent: boolean = false;

    constructor(isOpponent: boolean) {
        this.isOpponent = isOpponent;
        if (!isOpponent) this.randomize();
    }

    set(x: number, y: number, type: CellType) {
        this.board[x][y] = type;
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


