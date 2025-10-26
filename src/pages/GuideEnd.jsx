import { Link } from "react-router-dom"


function GuideEnd(){
    
    return(
        <div className="flex flex-row gap-3 justify-center items-center h-[100vh] w-[100vw] bg-[#3D3C3C]">
        <div className="text-center text-5xl">
            All done, continue to 
        </div>
        <Link to='/'><button type="button" className="py-2 px-3 bg-black rounded-md text-center text-white text-5xl">Pathfinder</button></Link>
        </div>
    )
} 
export default GuideEnd