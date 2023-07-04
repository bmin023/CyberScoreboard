import { useResetScores, useStartGame, useStopGame } from "../../Hooks/CtrlHooks";

interface IControlsProps {
    active: boolean;
}
const Controls: React.FC<IControlsProps> = ({ active }) => {
    const { startGame } = useStartGame();
    const { resetScores } = useResetScores();
    const { stopGame } = useStopGame();

    return (
        <div className="p-2 bg-slate-300 m-4 rounded-sm shadow-md pb-1 dark:bg-zinc-800">
            <h2 className="text-2xl text-center font-bold">Controls</h2>
            <div className="flex space-x-2">
                {active ? (
                    <button
                        className="flex-grow text-2xl bg-red-300 dark:bg-red-500 py-2 rounded shadow hover:shadow-lg active:shadow-none"
                        onClick={() => stopGame()}
                    >
                        Stop Game
                    </button>
                ) : (
                    <button
                        className="flex-grow text-2xl bg-green-300 dark:bg-emerald-500 py-2 rounded shadow hover:shadow-lg active:shadow-none"
                        onClick={() => startGame()}
                    >
                        Start Game
                    </button>
                )}
                <button
                    className="flex-grow text-2xl bg-blue-300 dark:bg-blue-500 py-2 rounded shadow hover:shadow-lg active:shadow-none"
                    onClick={() => resetScores()}
                >
                    Reset Scores
                </button>
            </div>
        </div>
    );
};

export default Controls;
