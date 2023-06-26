import { useEffect, useRef } from "react";
import { Link, useParams } from "react-router-dom";
import { useTeamInject, useUploadInject } from "../Hooks/CtrlHooks";

const TeamInject = () => {
    const { teamName, injectId } = useParams();
    const ref = useRef<HTMLDivElement>(null);
    const { inject, injectError, injectLoading } = useTeamInject(teamName, injectId);
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
            <p className="text-center text-xl font-mono">Start: {inject.desc.start} min, Duration: {inject.desc.duration} min</p>
            <div className="prose mx-5 md:mx-auto prose-slate dark:prose-invert" ref={ref} />
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
    return (
        <section className="p-5">
            <h2 className="text-3xl text-center font-extrabold m-2">
                Upload Files
            </h2>
            <form className="bg-slate-200 md:w-3/4 lg:w-1/2 mx-auto shadow-md p-3 rounded" method="post" encType="multipart/form-data" onSubmit={e => {
                e.preventDefault();
                const formData = new FormData(e.currentTarget);
                uploadInject(formData);
            }}>
                <div className="flex justify-center">
                    <label className="mr-2" htmlFor="file">Choose File to Upload</label>
                    <input
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
                        inject.desc.file_type.join(",")
                        : "Any"}</em>
                </div>
                <div className="flex justify-center mt-2">
                    <button ref={submitButton} className="bg-slate-300 py-2 px-3 rounded shadow-md text-xl font-bold text-center disabled:shadow-none disabled:text-zinc-100 hover:shadow-lg active:shadow-sm">Submit</button>
                </div>
            </form>
        </section>
    );
};

const InjectHistory = () => {
    const { teamName, injectId } = useParams();
    const { inject, injectLoading } = useTeamInject(teamName, injectId);
    if (injectLoading) return <div>Loading...</div>;
    return (
        <section className="p-5">
            <h2 className="text-3xl text-center font-extrabold m-2">
                Upload History
            </h2>
            <div className="bg-slate-200 md:w-3/4 lg:w-1/2 mx-auto shadow-md p-3 rounded">
                <ul>
                    {inject.history.map((response, i) => (
                        <li key={response.uuid} className="font-semibold">
                            Uploaded {response.filename} at {new Date(response.upload_time).toTimeString()}{response.late ? " (late)" : ""}
                        </li>
                    ))}
                </ul>
            </div>
        </section>
    );
};

export default TeamInject;
