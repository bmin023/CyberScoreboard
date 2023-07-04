import { useState } from "react";
import NaiveShrinker from "../../Components/NaiveShrinker";
import { Shrinker } from "../../Components/Shrinker";
import { useAddService, useAdminInfo, useDeleteService, useEditService, useTestService } from "../../Hooks/CtrlHooks";
import { Service } from "../../types";

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
                        {testServiceData.map((test) => (
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

export default Services;
