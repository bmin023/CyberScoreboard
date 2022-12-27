import { useRef, useState } from "react";
import NaiveShrinker from "../Components/NaiveShrinker";
import { Shrinker } from "../Components/Shrinker";
import Teams from "../Components/TeamAdmin";
import {
  useAddService,
  useAdminInfo,
  useDeleteService,
  useEditService,
  useLoadSave,
  useResetScores,
  useSaveData,
  useSaves,
  useStartGame,
  useStopGame,
  useTestService,
} from "../Hooks/CtrlHooks";
import { Service } from "../types";
import { formatDate } from "../util";

interface IServiceProps {
  service: Service;
}
const ServiceDisplay: React.FC<IServiceProps> = ({ service }) => {
  const { editService } = useEditService(service.name);
  const { deleteService } = useDeleteService(service.name);
  const { testService, testServiceData, testServiceUpdatedAt } = useTestService(
    service.name
  );
  const [name, setName] = useState(service.name);
  const [command, setCommand] = useState(service.command);
  const [multiplier, setMultiplier] = useState(service.multiplier);

  const hasChanged = () => {
    return (
      name !== service.name ||
      command !== service.command ||
      multiplier !== service.multiplier
    );
  };

  return (
    <div>
      <div className="flex flex-row mx-2 my-1">
        <Shrinker
          className="bg-slate-300 dark:bg-zinc-800 font-medium outline-none"
          value={name}
          onChange={(e) => setName(e)}
        />
        <p>:</p>
        <Shrinker
          className="bg-slate-300 dark:bg-zinc-800 px-1 font-mono outline-none"
          value={command}
          onChange={(e) => setCommand(e)}
        />
        <label className="ml-auto">Multiplier:</label>
        <input
          className="text-center w-10 dark:bg-zinc-800 bg-slate-300 mx-1 outline-none"
          type="number"
          value={multiplier}
          onChange={(e) =>
            setMultiplier(Math.max(Number.parseInt(e.target.value), 0))
          }
        />
        <button
          className="bg-slate-100 dark:bg-zinc-600 dark:disabled:bg-zinc-800 dark:disabled:bg-opacity-50 rounded-sm px-1 shadow-sm disabled:bg-slate-200 disabled:shadow-none disabled:text-gray-500 hover:shadow-lg active:shadow-none active:bg-slate-200"
          onClick={() => editService({ name, command, multiplier })}
          disabled={!hasChanged()}
        >
          Save
        </button>
        <button
          className="outline outline-1 text-zinc-500 hover:text-red-500 ml-1 rounded-sm px-1"
          onClick={() => deleteService()}
        >
          Delete
        </button>
        <button
          className="outline outline-1 text-zinc-500 hover:text-blue-500 ml-1 rounded-sm px-1"
          onClick={() => {
            if (hasChanged()) {
              editService(
                { name, command, multiplier },
                {
                  onSuccess: () => testService(),
                }
              );
            } else {
              testService();
            }
          }}
        >
          Test
        </button>
      </div>
      {testServiceData.length > 0 && (
        <details className="flex flex-row mx-2 my-1">
          <summary className="font-mono text-xs">
            Test at {new Date(testServiceUpdatedAt).toLocaleString()}
          </summary>
          <div>
            {testServiceData.map((test, i) => (
              <div className="flex text-sm mx-4 mt-1">
                <h4 className="" key={test.team}>
                  {test.team}:
                </h4>
                <p
                  className={
                    "mx-1 " + (test.up ? "text-green-500" : "text-red-500")
                  }
                >
                  {test.up ? "Up" : "Down"}
                </p>
                {test.message != "" && (
                  <p className="mx-1 font-mono">
                    stdout:
                    <span className="bg-slate-200 p-0.5 dark:bg-zinc-600 rounded-md shadow">
                      {test.message}
                    </span>
                  </p>
                )}
                {test.error != "" && (
                  <p className="mx-1 font-mono">
                    stderr:
                    <span className="bg-slate-200 p-0.5 dark:bg-zinc-600 rounded-md shadow">
                      {test.error}
                    </span>
                  </p>
                )}
              </div>
            ))}
          </div>
        </details>
      )}
    </div>
  );
};
const Services = () => {
  const { info } = useAdminInfo();
  const { addService } = useAddService();

  const onSubmit = (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    const formData = new FormData(e.currentTarget);
    const name = formData.get("name") as string;
    const command = formData.get("command") as string;
    const multiplier = Number.parseInt(formData.get("multiplier") as string);
    addService({ name, command, multiplier });
    // Reset form
    e.currentTarget.reset();
  };

  return (
    <div className="bg-slate-300 m-4 rounded-sm shadow-md pb-1 dark:bg-zinc-800">
      <h2 className="text-2xl text-center font-bold">Services</h2>
      {info.services.map((service) => (
        <ServiceDisplay service={service} key={service.name} />
      ))}
      <form onSubmit={onSubmit} className="flex flex-row mx-2 my-1">
        <NaiveShrinker
          placeholder="Name"
          name="name"
          className="bg-slate-300 font-medium dark:bg-zinc-800 outline-none"
        />
        <p>:</p>
        <NaiveShrinker
          placeholder="Command"
          name="command"
          className="bg-slate-300 px-1 font-mono dark:bg-zinc-800 outline-none"
        />
        <label className="ml-auto">Multiplier:</label>
        <input
          className="text-center w-10 bg-slate-300 mx-1 dark:bg-zinc-800 outline-none"
          type="number"
          name="multiplier"
          defaultValue={1}
        />
        <button className="outline outline-1 text-zinc-500 hover:text-green-400 ml-1 rounded-sm px-1">
          Add Service
        </button>
      </form>
    </div>
  );
};

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
            className="flex-grow text-2xl bg-red-300 py-2 rounded shadow hover:shadow-lg active:shadow-none"
            onClick={() => stopGame()}
          >
            Stop Game
          </button>
        ) : (
          <button
            className="flex-grow text-2xl bg-green-300 py-2 rounded shadow hover:shadow-lg active:shadow-none"
            onClick={() => startGame()}
          >
            Start Game
          </button>
        )}
        <button
          className="flex-grow text-2xl bg-blue-300 py-2 rounded shadow hover:shadow-lg active:shadow-none"
          onClick={() => resetScores()}
        >
          Reset Scores
        </button>
      </div>
    </div>
  );
};

const Saves = () => {
  const { saves, savesLoading, savesError } = useSaves();
  const { loadSave } = useLoadSave();
  const { saveData } = useSaveData();
  const inputRef = useRef<HTMLInputElement>();
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
        <details className="my-2 shadow-inner py-1">
          <summary>Autosaves</summary>
          <div className="pl-2">
            {saves.autosaves.map((save, i) => (
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
        {saves.saves.map((save, i) => (
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

const AdminPage = () => {
  const { info, infoLoading, infoError } = useAdminInfo();
  if (infoLoading)
    return (
      <div className="bg-slate-100 h-screen dark:bg-zinc-900 dark:text-zinc-100">
        <div>Loading...</div>
      </div>
    );
  if (infoError)
    return (
      <div className="bg-slate-100 h-screen dark:bg-zinc-900 dark:text-zinc-100">
        <p>We encountered an error. Probably Server.</p>
      </div>
    );
  return (
    <div className="py-1 bg-slate-100 h-screen dark:bg-zinc-900 dark:text-zinc-100">
      <h1 className="text-4xl text-center font-bold">Admin Page</h1>
      <Controls active={info.active} />
      <Services />
      <Teams />
      <Saves />
    </div>
  );
};

export default AdminPage;
