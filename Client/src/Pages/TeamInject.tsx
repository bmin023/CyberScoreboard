import { useEffect, useRef, useState } from "react";
import { Link, useParams } from "react-router-dom";
import { useTeamInject, useTime, useUploadInject } from "../Hooks/CtrlHooks";
import { formatDate } from "../util";

const TeamInject = () => {
    const { teamName, injectId } = useParams();
    const { time } = useTime(15000);
    const ref = useRef<HTMLDivElement>(null);
    const { inject, injectError, injectLoading } = useTeamInject(teamName, injectId);
    const [secondsSince, setSeconds] = useState(0);

    useEffect(() => {
        setSeconds(0);
    }, [time]);

    useEffect(() => {
        console.log(time);
        const interval = setInterval(() => {
            if (time.active) {
                setSeconds(secondsSince + 1);
            }
        }, 1000, 1000);
        return () => clearInterval(interval);
    }, [secondsSince, time.active]);


    const getTimeRemaining = () => {
        const high = time
        const endTime = inject.desc.start + inject.desc.duration;
        let minutes = endTime - high.minutes - 1;
        let seconds = 60 - (high.seconds + secondsSince);
        if ( seconds < 0 ) {
            seconds = 60 + seconds;
            minutes -= 1;
        }
        const hours = Math.floor(minutes / 60);
        minutes = minutes % 60;
        if (minutes < 0) return "Done";
        return `Due in ${hours > 0 ? `${hours}:`:''}${minutes}:${seconds < 10 ? "0" : ""}${seconds}`;
    };
    useEffect(() => {
        if (inject && ref.current) {
            ref.current.innerHTML = inject.html;
        }
    }, [inject, ref]);
    if (injectLoading)
        return (
            <div className="bg-slate-100 h-screen dark:bg-zinc-900 dark:text-zinc-100">
                <div>Loading...</div>
            </div>
        );
    if (injectError)
        return (
            <div className="bg-slate-100 h-screen dark:bg-zinc-900 dark:text-zinc-100">
                <p>
                    We encountered an error while retrieving the inject. It's possible this inject does not exists anymore. Did you use a stale link?
                </p>
            </div>
        );
    return (
        <div className="bg-slate-100 h-screen dark:bg-zinc-900 dark:text-zinc-100">
            <div className="flex">
                <Link className="underline m-1" to={`/team/${teamName}`}>Main Team Page</Link>
            </div>
            <h1 className="text-5xl text-center font-extrabold">
                {inject.desc.name}
            </h1>
            <p className="text-center text-xl font-mono">Start: {inject.desc.start} min{!inject.desc.sticky && `, Duration: ${inject.desc.duration} min`}</p>
            {!inject.desc.sticky &&
                <p className="text-center text-xl font-mono">{getTimeRemaining()}</p>
            }
            <div className="prose mx-5 mt-2 md:mx-auto prose-slate dark:prose-invert" ref={ref} />
            <UploadPane />
            <InjectHistory />
        </div>
    );
};

const UploadPane = () => {
    const { teamName, injectId } = useParams();
    const { inject } = useTeamInject(teamName, injectId);
    const { uploadInject } = useUploadInject(teamName, injectId);
    const fileInput = useRef<HTMLInputElement>(null);
    const hasFile = (): boolean => fileInput.current != null && fileInput.current.files != null && fileInput.current.files.length > 0;
    const submitButton = useRef<HTMLButtonElement>(null);
    useEffect(() => {
        if (submitButton.current)
            submitButton.current.disabled = !hasFile();
    }, [submitButton]);
    if (inject && inject.desc.file_type && inject.desc.file_type.length === 0) return <></>;
    return (
        <section className="p-5">
            <h2 className="text-3xl text-center font-extrabold m-2">
                Upload Files
            </h2>
            <form className="bg-slate-200 md:w-3/4 lg:w-1/2 mx-auto shadow-md p-3 rounded dark:bg-zinc-800" method="post" encType="multipart/form-data" onSubmit={e => {
                e.preventDefault();
                const formData = new FormData(e.currentTarget);
                uploadInject(formData);
            }}>
                <div className="flex justify-center">
                    <label className="mr-2" htmlFor="file">Choose File to Upload</label>
                    <input
                        className="file:bg-slate-300 dark:file:bg-zinc-900 dark:file:text-slate-50 file:rounded file:shadow-md file:font-bold file:text-center file:hover:shadow-lg file:active:shadow-sm"
                        onChange={_ => {
                            if (submitButton.current)
                                submitButton.current.disabled = !hasFile();
                        }}
                        ref={fileInput}
                        type="file"
                        id="file"
                        name="file"
                        accept={inject.desc.file_type ?
                            inject.desc.file_type.join(",")
                            : undefined}
                    />
                </div>
                <div className="flex justify-center">
                    <em className="mx-auto">File Type: {inject.desc.file_type ?
                        inject.desc.file_type.join(", ")
                        : "Any"}</em>
                </div>
                <div className="flex justify-center mt-2">
                    <button ref={submitButton} className="bg-slate-300 dark:bg-zinc-900 py-2 px-3 rounded shadow-md text-xl font-bold text-center disabled:shadow-none disabled:text-zinc-100 dark:disabled:text-zinc-500 hover:shadow-lg active:shadow-sm">Submit</button>
                </div>
            </form>
        </section>
    );
};

const InjectHistory = () => {
    const { teamName, injectId } = useParams();
    const { inject, injectLoading } = useTeamInject(teamName, injectId);
    if (injectLoading) return <div>Loading...</div>;
    if (inject.history.length === 0) return <></>;
    return (
        <section className="p-5">
            <h2 className="text-3xl text-center font-extrabold m-2">
                Upload History
            </h2>
            <div className="bg-slate-200 dark:bg-zinc-800 md:w-3/4 lg:w-1/2 mx-auto shadow-md p-3 rounded">
                <ul className="px-3 list-disc">
                    {inject.history.map((response) => (
                        <li key={response.uuid}>
                            Uploaded <span className="font-bold">{response.filename}</span> at {formatDate(new Date(response.upload_time))}<span className="text-red-500">{response.late ? " (Late)" : ""}</span>
                        </li>
                    ))}
                </ul>
            </div>
        </section>
    );
};

export default TeamInject;
