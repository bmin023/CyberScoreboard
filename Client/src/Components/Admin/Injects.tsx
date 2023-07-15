import { useEffect, useState } from "react";
import { useAdminAddInject, useAdminDeleteInject, useAdminEditInject, useAdminInjects } from "../../Hooks/CtrlHooks";
import { Inject, SideEffect } from "../../types";

const Injects = () => {
    const { injects, injectsLoading } = useAdminInjects();
    const { editInject } = useAdminEditInject();
    const { deleteInject } = useAdminDeleteInject();
    const { addInject } = useAdminAddInject();


    if (injectsLoading)
        return (
            <div className="p-2 bg-slate-300 m-4 rounded-sm shadow-md pb-1 dark:bg-zinc-800">
                <h2 className="text-2xl text-center font-bold">Injects</h2>
                <div className="text-center">Loading...</div>
            </div>
        );
    return (
        <div className="p-2 bg-slate-300 m-4 rounded-sm shadow-md pb-1 dark:bg-zinc-800">
            <h2 className="text-2xl text-center font-bold">Injects</h2>
            {injects.map((inject) => (
                <div key={inject.uuid} className="m-1">
                    <details>
                        <summary className="font-bold">{inject.name}{inject.completed ? " (Completed)" : ""}</summary>
                        <div className="pl-7">
                            <InjectEditor inject={inject} onSubmit={(submittable) => {
                                editInject(submittable);
                            }} onDelete={(uuid)=>deleteInject(uuid)} />
                        </div>
                    </details>
                </div>
            ))}
            <div className="m-1">
                <details>
                    <summary className="font-bold">Add Inject</summary>
                    <div className="pl-7">
                        <InjectEditor inject={{ uuid: "newinject", name: "New Inject", start: 0, duration: 0, file_type: null, markdown: "", side_effects: [], completed: false, sticky: false }} onSubmit={(submittable) => {
                            addInject(submittable);
                        }} submitText="Add" />
                    </div>
                </details>
            </div>
        </div>
    );
};

interface InjectEditorProps {
    inject: Inject;
    submitText?: string;
    onDelete?: (uuid: string) => void;
    onSubmit?: (inject: Inject) => void;
}

const InjectEditor = ({ inject, onSubmit, submitText, onDelete }: InjectEditorProps) => {
    const [name, setName] = useState(inject.name);
    const [start, setStart] = useState(inject.start);
    const [duration, setDuration] = useState(inject.duration);
    const [fileTypes, setFileTypes] = useState(inject.file_type);
    const [markdown, setMarkdown] = useState(inject.markdown);
    const [sideEffects, setSideEffects] = useState(inject.side_effects);
    const [sideEffectData, setSideEffectData] = useState(inject.side_effects.map((_, i) => ({ id: i })));
    const [sticky, setSticky] = useState(inject.sticky);
    const submit = () => {
        if (!onSubmit) return;
        const submittable: Inject = {
            uuid: inject.uuid,
            name,
            start,
            duration,
            file_type: fileTypes,
            markdown,
            side_effects: sideEffects,
            completed: inject.completed,
            sticky,
        };
        onSubmit(submittable);
    };

    const hasChanged = () => {
        const sideEffectsEqual = sideEffects.length === inject.side_effects.length && sideEffects.every((sideEffect, i) => {
            const myKey = Object.keys(sideEffect)[0];
            const theirKey = Object.keys(inject.side_effects[i])[0];
            return myKey === theirKey && JSON.stringify(sideEffect[myKey]) === JSON.stringify(inject.side_effects[i][theirKey]);
        });
        return name != inject.name ||
            sticky != inject.sticky ||
            start != inject.start ||
            duration != inject.duration ||
            fileTypes != inject.file_type ||
            markdown != inject.markdown ||
            !sideEffectsEqual;
    }

    const addFileType = (e: React.FormEvent<HTMLFormElement>) => {
        e.preventDefault();
        const formData = new FormData(e.currentTarget);
        let filetype = formData.get("filetype") as string;
        // clear input
        e.currentTarget.reset();
        if (filetype.toLowerCase() === "any" || filetype === "*") {
            setFileTypes(null);
            return;
        }
        if (filetype[0] !== ".") filetype = "." + filetype;
        if (!fileTypes) {
            setFileTypes([filetype]);
            return;
        }
        setFileTypes([...fileTypes, filetype]);
    }
    const removeFileType = (filetype: string) => {
        if (filetype.toLowerCase() === "any") {
            setFileTypes([]);
            return;
        }
        if (!fileTypes) return;
        setFileTypes(fileTypes.filter(ft => ft !== filetype));
    }

    const editSideEffect = (sideEffect: SideEffect, id: number) => {
        const index = sideEffectData.findIndex((data) => data.id === id);
        const newSideEffects = [...sideEffects];
        newSideEffects[index] = sideEffect;
        setSideEffects(newSideEffects);
    }

    const addSideEffect = () => {
        let newId = Math.round(Math.random() * 1000);
        while (sideEffectData.findIndex((_, i) => i === newId) !== -1) {
            newId = Math.round(Math.random() * 1000);
        }
        setSideEffectData([...sideEffectData, { id: newId }]);
        setSideEffects([...sideEffects, {
            Type: "Value"
        }]);
    }

    const removeSideEffect = (id: number) => {
        const index = sideEffectData.findIndex((data) => data.id === id);
        if (index === -1) return;
        setSideEffects(sideEffects.filter((_, i) => i !== index));
        setSideEffectData(sideEffectData.filter((_, i) => i !== index));
    }
    return (
        <div className="flex flex-col gap-2">
            <div className="flex gap-3">
                <label className="font-medium">Inject Name</label>
                <input className="flex-grow rounded bg-slate-100 px-1 shadow font-medium dark:bg-zinc-700" value={name} onChange={e => setName(e.currentTarget.value)} />
            </div>
            <div className="flex gap-4">
                <label className="font-medium">Start
                    <input min="0" step="1" type="number" className="w-20 mx-2 rounded bg-slate-100 px-1 shadow font-medium dark:bg-zinc-700" value={start} onChange={e => setStart(parseInt(e.currentTarget.value))} />
                    <span className="font-light">min</span>
                </label>
                <label className="font-medium">Duration
                    <input type="number" className="w-20 mx-2 rounded bg-slate-100 px-1 shadow font-medium dark:bg-zinc-700 disabled:bg-opacity-25 disabled:text-opacity-25 text-black dark:text-white" disabled={sticky} value={duration} onChange={e => setDuration(parseInt(e.currentTarget.value))} />
                    <span className="font-light">min</span>
                </label>
                <label className="font-medium">Sticky
                    <input type="checkbox" className="mx-2 rounded bg-slate-100 px-1 shadow font-medium dark:bg-zinc-700 " checked={sticky} onChange={() => setSticky(!sticky)} />
                </label>
            </div>
            <div className="flex gap-2">
                <label className="font-medium">Accepted File Types</label>
                {!fileTypes && <FileTypeBadge onClick={() => removeFileType("any")} filetype="Any" />}
                {fileTypes && (fileTypes.length > 0 ?
                    fileTypes.map((fileType) => (
                        <FileTypeBadge onClick={() => removeFileType(fileType)} filetype={fileType} />
                    ))
                    : <FileTypeBadge onClick={() => setFileTypes(null)} filetype="None" />
                )}
                <form onSubmit={addFileType} className="focus-within:outline outline-1 bg-slate-100 rounded shadow-md px-1 dark:bg-zinc-700">
                    <input pattern="([Aa]ny|\.?\w+|\*)" name="filetype" placeholder="Add (e.g. .txt, .docx, any)" className="w-48 px-1 flex-grow bg-transparent outline-none" />
                    <input type="submit" value="+" />
                </form>
            </div>
            <details>
                <summary className="font-medium">Inject Description</summary>
                <textarea className="h-80 resize-y w-full rounded bg-slate-100 py-1 px-2 shadow font-medium dark:bg-zinc-700" value={markdown} onChange={e => setMarkdown(e.currentTarget.value)} />
            </details>
            <details>
                <summary className="font-medium">Side Effects</summary>
                <div className="pl-5">
                    {sideEffects.map((side_effect, i) =>
                        <SideEffectEditor
                            key={sideEffectData[i].id}
                            sideEffect={side_effect}
                            onChange={se => editSideEffect(se, sideEffectData[i].id)}
                            onDelete={() => removeSideEffect(sideEffectData[i].id)}
                        />
                    )}
                    <button onClick={addSideEffect} className="bg-slate-200 w-full px-1 rounded shadow-sm hover:shadow-md active:shadow-sm dark:bg-zinc-700">Add Side Effect</button>
                    <div className="font-extralight">
                        Please know what you are doing if you are editing these. It is still suggested you use the config file.
                    </div>
                </div>
            </details>
            <div className="flex gap-2">
                <button disabled={!hasChanged()} onClick={submit} className="bg-slate-200 w-full px-1 rounded shadow-sm hover:shadow-md active:shadow-sm disabled:shadow-none disabled:bg-opacity-50 disabled:text-slate-400 dark:bg-zinc-600">{submitText ?? "Save"}</button>
                { onDelete && <button onClick={()=>onDelete(inject.uuid)} className="bg-slate-200 w-full px-1 rounded shadow-sm hover:shadow-md active:shadow-sm dark:bg-zinc-600">Delete</button>}
            </div>
        </div>
    );
}


const FileTypeBadge = ({ filetype, onClick }: { filetype: string, onClick: () => void }) => {
    return (
        <button onClick={onClick} className="hover:shadow-md active:shadow-none bg-slate-100 px-1 rounded shadow-sm dark:bg-zinc-700">{filetype}</button>
    );
}

interface SideEffectEditorProps {
    sideEffect: SideEffect
    onChange?: (sideEffect: SideEffect) => void
    onDelete?: () => void
}

export const SideEffectEditor = ({ sideEffect, onChange, onDelete }: SideEffectEditorProps) => {
    const [key, setKey] = useState(Object.keys(sideEffect)[0]);
    const [value, setValue] = useState(JSON.stringify(sideEffect[key]));
    const [message, setMessage] = useState(null as string | null);

    useEffect(() => {
        if (onChange) {
            try {
                let v = JSON.parse(value);
                onChange({ [key]: v });
                setMessage(null);
            } catch (e) {
                setMessage("parse error");
                return;
            }
        }
    }, [value, key]);
    return (
        <div className="flex">
            <input className="outline-none bg-slate-300 font-bold w-[7.5rem] dark:bg-zinc-800" value={key} onChange={(e) => { setKey(e.currentTarget.value); }} /><b>:</b>
            <input className="outline-none flex-grow bg-slate-300 px-1 dark:bg-zinc-800" value={value} onChange={e => {
                setValue(e.currentTarget.value);
            }} />
            {message && <span className="text-red-500">{message}</span>}
            <button className="px-1" onClick={onDelete}>X</button>
        </div>
    )
}

export default Injects;
