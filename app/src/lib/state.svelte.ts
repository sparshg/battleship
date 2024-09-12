
export type Phase = 'placement' | 'battle' | 'gameover';
export type CellType = 'empty' | 'ship' | 'hit' | 'miss';
export type Board = Array<Array<CellType>>;

export class State {
    phase: Phase = $state('placement');
}
