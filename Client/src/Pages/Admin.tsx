import Teams from "../Components/Admin/TeamAdmin";
import {
    useAdminInfo,
} from "../Hooks/CtrlHooks";
import Saves from "../Components/Admin/Saves";
import Controls from "../Components/Admin/Controls";
import Services from "../Components/Admin/Services";
import Injects from "../Components/Admin/Injects";


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
            <Injects />
            <Saves />
        </div>
    );
};

export default AdminPage;
