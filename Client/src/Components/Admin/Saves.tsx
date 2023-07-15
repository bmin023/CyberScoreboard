import { useState } from "react";
import { useLoadSave, useSaveData, useSaves } from "../../Hooks/CtrlHooks";

const Saves = () => {
    const { saves, savesLoading, savesError } = useSaves();
    const { loadSave } = useLoadSave();
    const { saveData } = useSaveData();
    const [selected, setSelected] = useState<string | null>(null);
    if (savesLoading) return <div>Loading...</div>;
    if (savesError) return <div>We encountered an error. Probably Server.</div>;

    const buttonStyle = (name: string) =>
        name === selected
            ? "bg-slate-200 p-1 rounded-md w-full my-0.5 shadow-md dark:bg-zinc-500"
            : "bg-slate-200 p-1 rounded-md w-full my-0.5 shadow-md dark:bg-zinc-500 opacity-50";

    const canLoad = (save: string) => {
        return (
            saves.saves.map((v) => v.name).includes(save) ||
            saves.autosaves.map((v) => v.name).includes(save)
        );
    }

    return (
        <div className="p-2 bg-slate-300 m-4 rounded-sm shadow-md pb-1 dark:bg-zinc-800">
            <h2 className="text-2xl text-center font-bold">Saves</h2>
            <div className="bg-slate-400 p-2 rounded shadow-lg dark:bg-zinc-700">
                <details className="shadow-inner py-1">
                    <summary>Autosaves</summary>
                    <div className="pl-2">
                        {saves.autosaves.map((save) => (
                            <button
                                className={buttonStyle(save.name)}
                                onClick={() => setSelected(save.name)}
                                key={save.name}
                            >
                                {save.name}{" "}
                                <span className="font-light text-sm">
                                    {new Date(save.timestamp).toLocaleString()}
                                </span>
                            </button>
                        ))}
                    </div>
                </details>
                {saves.saves.map((save) => (
                    <button
                        className={buttonStyle(save.name)}
                        onClick={() => setSelected(save.name)}
                        key={save.name}
                    >
                        {save.name}{" "}
                        <span className="font-light text-sm">
                            {new Date(save.timestamp).toLocaleString()}
                        </span>
                    </button>
                ))}
            </div>
            {selected && canLoad(selected) && (
                <button
                    onClick={() => loadSave({ name: selected })}
                    className="w-full bg-slate-100 mt-2 p-2 rounded shadow-lg font-medium dark:bg-zinc-700"
                >
                    Load {selected}
                </button>
            )}
            <div className="flex mt-2">
                <input
                    className="flex-grow p-2 rounded bg-slate-100 shadow-lg font-medium dark:bg-zinc-700"
                    value={selected ? selected : ""}
                    onChange={(e) => setSelected(e.target.value)}
                />
                <button
                    onClick={() => saveData({ name: selected ? selected : "unknown" })}
                    className="ml-2 w-14 bg-slate-200 p-2 rounded shadow-lg font-bold dark:bg-zinc-700"
                >
                    Save
                </button>
            </div>
        </div>
    );
};

export default Saves;
